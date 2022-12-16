use crate::db::operations::update_tx;
use crate::error::PaymentError;
use crate::error::*;
use crate::{err_create, err_custom_create, err_from};
use sqlx::SqliteConnection;
use std::str::FromStr;
use std::time::Duration;
use web3::transports::Http;
use web3::types::{Address, U256};
use web3::Web3;

use crate::eth::{get_eth_addr_from_secret, get_transaction_count};
use crate::model::Web3TransactionDao;
use crate::setup::PaymentSetup;
use crate::transaction::check_transaction;
use crate::transaction::find_receipt;
use crate::transaction::send_transaction;
use crate::transaction::sign_transaction;
use crate::utils::u256_to_rust_dec;

#[derive(Debug)]
pub enum ProcessTransactionResult {
    Confirmed,
    NeedRetry(String),
    InternalError(String),
    Unknown,
}

#[allow(dead_code)]
pub async fn get_provider(url: &str) -> Result<Web3<Http>, PaymentError> {
    let transport = web3::transports::Http::new(url).map_err(err_from!())?;
    let web3 = web3::Web3::new(transport);
    Ok(web3)
}

pub async fn process_transaction(
    conn: &mut SqliteConnection,
    web3_tx_dao: &mut Web3TransactionDao,
    payment_setup: &PaymentSetup,
    wait_for_confirmation: bool,
) -> Result<ProcessTransactionResult, PaymentError> {
    const CHECKS_UNTIL_NOT_FOUND: u64 = 5;

    let wait_duration = Duration::from_secs(payment_setup.process_sleep);

    let chain_id = web3_tx_dao.chain_id;
    let chain_setup = payment_setup.get_chain_setup(chain_id).map_err(|_e| {
        err_create!(TransactionFailedError::new(&format!(
            "Failed to get chain setup for chain id: {}",
            chain_id
        )))
    })?;

    let web3 = payment_setup.get_provider(chain_id).map_err(|_e| {
        err_create!(TransactionFailedError::new(&format!(
            "Failed to get provider for chain id: {}",
            chain_id
        )))
    })?;
    let from_addr = Address::from_str(&web3_tx_dao.from_addr)
        .map_err(|_e| err_create!(TransactionFailedError::new("Failed to parse from_addr")))?;

    let private_key = payment_setup
        .secret_keys
        .iter()
        .find(|sk| get_eth_addr_from_secret(sk) == from_addr)
        .ok_or(err_create!(TransactionFailedError::new(&format!(
            "Failed to find private key for address: {}",
            from_addr
        ))))?;

    let transaction_nonce = if let Some(nonce) = web3_tx_dao.nonce {
        nonce
    } else {
        let nonce = get_transaction_count(from_addr, web3, false)
            .await
            .map_err(err_from!())? as i64;
        web3_tx_dao.nonce = Some(nonce);
        nonce
    };

    //this block is optional, just to warn user about low gas
    let perform_balance_check = true;
    if perform_balance_check {
        let gas_balance = web3
            .eth()
            .balance(from_addr, None)
            .await
            .map_err(err_from!())?;
        let expected_gas_balance = chain_setup.max_fee_per_gas
            * U256::from(chain_setup.gas_left_warning_limit);
        if gas_balance < expected_gas_balance {
            let msg = if gas_balance.is_zero() {
                format!("Account {} gas balance", chain_setup.currency_symbol)
            } else {
                format!(
                    "Account {} gas balance is very low",
                    chain_setup.currency_symbol
                )
            };

            log::warn!(
                "{} on chain {}, account: {:?}, gas_balance: {}, expected_gas_balance: {}",
                msg,
                chain_id,
                from_addr,
                u256_to_rust_dec(gas_balance, Some(18)).map_err(err_from!())?,
                u256_to_rust_dec(expected_gas_balance, Some(18)).map_err(err_from!())?
            );
        }
    }

    //timeout transaction when it is not confirmed after transaction_timeout seconds
    if let Some(first_processed) = web3_tx_dao.first_processed {
        let now = chrono::Utc::now();
        let diff = now - first_processed;
        if diff.num_seconds() < -10 {
            return Ok(ProcessTransactionResult::NeedRetry(
                "Time changed".to_string(),
            ));
        }
        if diff.num_seconds() > chain_setup.transaction_timeout as i64 {
            log::warn!("Transaction timeout for tx id: {}", web3_tx_dao.id);
            //return Ok(ProcessTransactionResult::NeedRetry("Timeout".to_string()));
        }
    } else {
        web3_tx_dao.first_processed = Some(chrono::Utc::now());
        update_tx(conn, web3_tx_dao).await.map_err(err_from!())?;
    }

    if web3_tx_dao.signed_raw_data.is_none() {
        log::info!("Checking transaction {}", web3_tx_dao.id);
        match check_transaction(web3, web3_tx_dao).await {
            Ok(_) => {}
            Err(err) => {
                let err_msg = format!("{}", err);
                if err_msg
                    .to_lowercase()
                    .contains("insufficient funds for transfer")
                {
                    log::error!(
                        "Insufficient {} for tx id: {}",
                        chain_setup.currency_symbol,
                        web3_tx_dao.id
                    );
                    return Err(err);
                }
                log::error!("Error while checking transaction: {}", err);
                return Err(err);
            }
        }
        log::debug!("web3_tx_dao after check_transaction: {:?}", web3_tx_dao);
        sign_transaction(web3, web3_tx_dao, private_key).await?;
        update_tx(conn, web3_tx_dao).await.map_err(err_from!())?;
    }

    if web3_tx_dao.broadcast_date.is_none() {
        log::info!(
            "Sending transaction {} with nonce {}",
            web3_tx_dao.id,
            transaction_nonce
        );
        send_transaction(web3, web3_tx_dao).await?;
        web3_tx_dao.broadcast_count += 1;
        update_tx(conn, web3_tx_dao).await.map_err(err_from!())?;
        log::info!(
            "Transaction {} sent, tx hash: {}",
            web3_tx_dao.id,
            web3_tx_dao.tx_hash.clone().unwrap_or_default()
        );
    }

    if web3_tx_dao.confirm_date.is_some() {
        log::info!("Transaction already confirmed {}", web3_tx_dao.id);
        return Ok(ProcessTransactionResult::Confirmed);
    }

    let mut tx_not_found_count = 0;
    loop {
        log::info!(
            "Checking latest nonce tx: {}, expected nonce: {}",
            web3_tx_dao.id,
            transaction_nonce + 1
        );
        let latest_nonce = get_transaction_count(from_addr, web3, false)
            .await
            .map_err(err_from!())?;
        let current_block_number = web3
            .eth()
            .block_number()
            .await
            .map_err(err_from!())?
            .as_u64();
        if latest_nonce
            > web3_tx_dao
                .nonce
                .map(|n| n as u64)
                .ok_or_else(|| err_custom_create!("Nonce not found"))?
        {
            let res = find_receipt(web3, web3_tx_dao).await?;
            if res {
                if let Some(block_number) = web3_tx_dao.block_number.map(|n| n as u64) {
                    log::info!(
                        "Receipt found: tx {} tx_hash: {}",
                        web3_tx_dao.id,
                        web3_tx_dao.tx_hash.clone().unwrap_or_default()
                    );
                    if block_number + chain_setup.confirmation_blocks <= current_block_number {
                        web3_tx_dao.confirm_date = Some(chrono::Utc::now());
                        log::info!(
                            "Transaction confirmed: tx: {} tx_hash: {}",
                            web3_tx_dao.id,
                            web3_tx_dao.tx_hash.clone().unwrap_or_default()
                        );
                        break;
                    } else {
                        log::info!("Waiting for confirmations: tx: {}. Current block {}, expected at least: {}", web3_tx_dao.id, current_block_number, block_number + chain_setup.confirmation_blocks);
                    }
                } else {
                    return Err(err_custom_create!(
                        "Block number not found on dao for tx: {}",
                        web3_tx_dao.id
                    ));
                }
            } else {
                tx_not_found_count += 1;
                log::debug!("Receipt not found: {:?}", web3_tx_dao.tx_hash);
                if tx_not_found_count >= CHECKS_UNTIL_NOT_FOUND {
                    return Ok(ProcessTransactionResult::NeedRetry(
                        "No receipt".to_string(),
                    ));
                }
            }
        } else {
            log::info!(
                "Latest nonce is not yet reached: {} vs {}",
                latest_nonce,
                transaction_nonce + 1
            );
        }
        log::info!(
            "Checking pending nonce tx: {}, expected nonce: {}",
            web3_tx_dao.id,
            transaction_nonce + 1
        );
        let pending_nonce = get_transaction_count(from_addr, web3, true)
            .await
            .map_err(err_from!())?;
        if pending_nonce
            <= web3_tx_dao
                .nonce
                .map(|n| n as u64)
                .ok_or_else(|| err_custom_create!("Nonce not found"))?
        {
            // this resend is safe because all tx data is the same,
            // it's just attempt of sending the same transaction
            log::warn!(
                "Resend because pending nonce too low. tx: {} tx_hash: {:?}",
                web3_tx_dao.id,
                web3_tx_dao.tx_hash.clone().unwrap_or_default()
            );
            send_transaction(web3, web3_tx_dao).await?;
            web3_tx_dao.broadcast_count += 1;
            update_tx(conn, web3_tx_dao).await.map_err(err_from!())?;
            tokio::time::sleep(wait_duration).await;
            continue;
        }
        if !wait_for_confirmation {
            return Ok(ProcessTransactionResult::Unknown);
        }
        tokio::time::sleep(wait_duration).await;
    }
    log::debug!("web3_tx_dao after confirmation: {:?}", web3_tx_dao);
    Ok(ProcessTransactionResult::Confirmed)
}
