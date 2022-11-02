use crate::db::operations::get_all_processed_transactions;
use secp256k1::SecretKey;
use std::error;
use std::str::FromStr;
use std::time::Duration;
use web3::transports::Http;
use web3::types::Address;
use web3::Web3;

use crate::eth::get_transaction_count;
use crate::model::Web3TransactionDao;
use crate::transaction::check_transaction;
use crate::transaction::find_receipt;
use crate::transaction::send_transaction;
use crate::transaction::sign_transaction;

#[derive(Debug)]
pub enum ProcessTransactionResult {
    Confirmed,
    NeedRetry,
    Unknown,
}

#[allow(dead_code)]
pub async fn get_provider(url: &str) -> Result<Web3<Http>, Box<dyn error::Error>> {
    let prov_url = url;
    let transport = web3::transports::Http::new(&prov_url)?;
    let web3 = web3::Web3::new(transport);
    Ok(web3)
}

pub async fn process_transaction(
    web3_tx_dao: &mut Web3TransactionDao,
    web3: &Web3<Http>,
    secret_key: &SecretKey,
    wait_for_confirmation: bool,
) -> Result<ProcessTransactionResult, Box<dyn error::Error>> {
    const CHECKS_UNTIL_NOT_FOUND: u64 = 5;
    const CONFIRMED_BLOCKS: u64 = 0;

    let _chain_id = web3_tx_dao.chain_id;
    let from_addr = Address::from_str(&web3_tx_dao.from_addr)?;
    if web3_tx_dao.nonce.is_none() {
        let nonce = get_transaction_count(from_addr, &web3, false).await?;
        web3_tx_dao.nonce = Some(nonce as i64);
    }
    if web3_tx_dao.signed_raw_data.is_none() {
        check_transaction(&web3, web3_tx_dao).await?;

        println!("web3_tx_dao after check_transaction: {:?}", web3_tx_dao);
        sign_transaction(&web3, web3_tx_dao, &secret_key).await?;
    }

    if web3_tx_dao.broadcast_date.is_none() {
        send_transaction(&web3, web3_tx_dao).await?;
    }

    if web3_tx_dao.confirmed_date.is_some() {
        return Ok(ProcessTransactionResult::Confirmed);
    }

    let mut tx_not_found_count = 0;
    loop {
        let pending_nonce = get_transaction_count(from_addr, &web3, true).await?;
        if pending_nonce
            <= web3_tx_dao
                .nonce
                .map(|n| n as u64)
                .ok_or("Nonce not found")?
        {
            println!(
                "Resend because pending nonce too low: {:?}",
                web3_tx_dao.tx_hash
            );
            send_transaction(&web3, web3_tx_dao).await?;
            tokio::time::sleep(Duration::from_secs(1)).await;
            continue;
        }
        let latest_nonce = get_transaction_count(from_addr, &web3, false).await?;
        let current_block_number = web3.eth().block_number().await?.as_u64();
        if latest_nonce
            > web3_tx_dao
                .nonce
                .map(|n| n as u64)
                .ok_or("Nonce not found")?
        {
            let res = find_receipt(&web3, web3_tx_dao).await?;
            if res {
                if let Some(block_number) = web3_tx_dao.block_number.map(|n| n as u64) {
                    println!("Receipt found: {:?}", web3_tx_dao.tx_hash);
                    if block_number + CONFIRMED_BLOCKS <= current_block_number {
                        web3_tx_dao.confirmed_date = Some(chrono::Utc::now());
                        println!("Transaction confirmed: {:?}", web3_tx_dao.tx_hash);
                        break;
                    }
                }
            } else {
                tx_not_found_count += 1;
                println!("Receipt not found: {:?}", web3_tx_dao.tx_hash);
                if tx_not_found_count >= CHECKS_UNTIL_NOT_FOUND {
                    return Ok(ProcessTransactionResult::NeedRetry);
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
