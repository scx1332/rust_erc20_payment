use secp256k1::SecretKey;

use crate::db::operations::update_tx;
use crate::error::PaymentError;
use sqlx::SqliteConnection;
use std::str::FromStr;
use std::time::Duration;
use web3::transports::Http;
use web3::types::Address;
use web3::Web3;

use crate::eth::get_transaction_count;
use crate::model::Web3TransactionDao;
use crate::setup::PaymentSetup;
use crate::transaction::check_transaction;
use crate::transaction::find_receipt;
use crate::transaction::send_transaction;
use crate::transaction::sign_transaction;

#[derive(Debug)]
pub enum ProcessTransactionResult {
    Confirmed,
    NeedRetry(String),
    InternalError(String),
    Unknown,
}

#[allow(dead_code)]
pub async fn get_provider(url: &str) -> Result<Web3<Http>, PaymentError> {
    let prov_url = url;
    let transport = web3::transports::Http::new(&prov_url)?;
    let web3 = web3::Web3::new(transport);
    Ok(web3)
}

pub async fn process_transaction(
    conn: &mut SqliteConnection,
    web3_tx_dao: &mut Web3TransactionDao,
    payment_setup: &PaymentSetup,
    secret_key: &SecretKey,
    wait_for_confirmation: bool,
) -> Result<ProcessTransactionResult, PaymentError> {
    const CHECKS_UNTIL_NOT_FOUND: u64 = 5;
    const CONFIRMED_BLOCKS: u64 = 0;

    let chain_id = web3_tx_dao.chain_id;
    let chain_setup = payment_setup.get_chain_setup(chain_id)?;
    let web3 = payment_setup.get_provider(chain_id)?;
    let from_addr = Address::from_str(&web3_tx_dao.from_addr)
        .map_err(|_e| PaymentError::ParsingError("Failed to parse from_addr".to_string()))?;
    if web3_tx_dao.nonce.is_none() {
        let nonce = get_transaction_count(from_addr, &web3, false).await?;
        web3_tx_dao.nonce = Some(nonce as i64);
    }

    //timeout transaction when it is not confirmed after transaction_timeout seconds
    if let Some(signed_date) = web3_tx_dao.signed_date {
        let now = chrono::Utc::now();
        let diff = now - signed_date;
        if diff.num_seconds() < -10 {
            return Ok(ProcessTransactionResult::NeedRetry("Time changed".to_string()));
        }
        if diff.num_seconds() > chain_setup.transaction_timeout as i64 {
            return Ok(ProcessTransactionResult::NeedRetry("Timeout".to_string()));
        }
    }

    if web3_tx_dao.signed_raw_data.is_none() {
        match check_transaction(&web3, web3_tx_dao).await {
            Ok(_) => {}
            Err(err) => {
                log::error!("Error while checking transaction: {}", err);
                return Err(err);
            }
        }

        println!("web3_tx_dao after check_transaction: {:?}", web3_tx_dao);
        sign_transaction(&web3, web3_tx_dao, &secret_key).await?;
        update_tx(conn, web3_tx_dao).await?;
    }

    if web3_tx_dao.broadcast_date.is_none() {
        send_transaction(&web3, web3_tx_dao).await?;
        web3_tx_dao.broadcast_count += 1;
        update_tx(conn, web3_tx_dao).await?;
    }

    if web3_tx_dao.confirm_date.is_some() {
        return Ok(ProcessTransactionResult::Confirmed);
    }


    let mut tx_not_found_count = 0;
    loop {
        let pending_nonce = get_transaction_count(from_addr, &web3, true).await?;
        if pending_nonce
            <= web3_tx_dao
                .nonce
                .map(|n| n as u64)
                .ok_or(PaymentError::OtherError("Nonce not found".to_string()))?
        {
            println!(
                "Resend because pending nonce too low: {:?}",
                web3_tx_dao.tx_hash
            );
            send_transaction(&web3, web3_tx_dao).await?;
            web3_tx_dao.broadcast_count += 1;
            update_tx(conn, web3_tx_dao).await?;
            tokio::time::sleep(Duration::from_secs(1)).await;
            continue;
        }
        let latest_nonce = get_transaction_count(from_addr, &web3, false).await?;
        let current_block_number = web3.eth().block_number().await?.as_u64();
        if latest_nonce
            > web3_tx_dao
                .nonce
                .map(|n| n as u64)
                .ok_or(PaymentError::OtherError("Nonce not found".to_string()))?
        {
            let res = find_receipt(&web3, web3_tx_dao).await?;
            if res {
                if let Some(block_number) = web3_tx_dao.block_number.map(|n| n as u64) {
                    log::debug!("Receipt found: {:?}", web3_tx_dao.tx_hash);
                    if block_number + CONFIRMED_BLOCKS <= current_block_number {
                        web3_tx_dao.confirm_date = Some(chrono::Utc::now());
                        log::debug!("Transaction confirmed: {:?}", web3_tx_dao.tx_hash);
                        break;
                    }
                }
            } else {
                tx_not_found_count += 1;
                log::debug!("Receipt not found: {:?}", web3_tx_dao.tx_hash);
                if tx_not_found_count >= CHECKS_UNTIL_NOT_FOUND {
                    return Ok(ProcessTransactionResult::NeedRetry("No receipt".to_string()));
                }
            }
        }
        if !wait_for_confirmation {
            return Ok(ProcessTransactionResult::Unknown);
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    println!("web3_tx_dao after confirmation: {:?}", web3_tx_dao);
    Ok(ProcessTransactionResult::Confirmed)
}
