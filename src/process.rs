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
}

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
) -> Result<ProcessTransactionResult, Box<dyn error::Error>> {
    const CHECKS_UNTIL_NOT_FOUND: u64 = 5;
    const CONFIRMED_BLOCKS: u64 = 0;

    let _chain_id = web3_tx_dao.chain_id;
    let from_addr = Address::from_str(&web3_tx_dao.from)?;
    let nonce = get_transaction_count(from_addr, &web3, false).await?;

    println!("nonce: {}", nonce);

    println!("web3_tx_dao: {:?}", web3_tx_dao);

    web3_tx_dao.nonce = Some(nonce);
    check_transaction(&web3, web3_tx_dao).await?;

    println!("web3_tx_dao after check_transaction: {:?}", web3_tx_dao);
    sign_transaction(&web3, web3_tx_dao, &secret_key).await?;

    println!("web3_tx_dao after sign_transaction: {:?}", web3_tx_dao);
    send_transaction(&web3, web3_tx_dao).await?;

    println!("web3_tx_dao after send_transaction: {:?}", web3_tx_dao);
    let mut tx_not_found_count = 0;
    loop {
        let pending_nonce = get_transaction_count(from_addr, &web3, true).await?;
        if pending_nonce <= web3_tx_dao.nonce.ok_or("Nonce not found")? {
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
        if latest_nonce > web3_tx_dao.nonce.ok_or("Nonce not found")? {
            let res = find_receipt(&web3, web3_tx_dao).await?;
            if res {
                if let Some(block_number) = web3_tx_dao.block_number {
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
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    println!("web3_tx_dao after confirmation: {:?}", web3_tx_dao);
    Ok(ProcessTransactionResult::Confirmed)
}
