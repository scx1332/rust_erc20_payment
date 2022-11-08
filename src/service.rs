use std::collections::HashMap;
use std::str::FromStr;

use crate::contracts::MULTI_ERC20_GOERLI;
use crate::db::operations::{
    find_allowance, get_allowance_by_tx, get_pending_token_transfers, get_token_transfers_by_tx,
    get_transactions_being_processed, insert_allowance, insert_tx, update_allowance,
    update_token_transfer, update_tx,
};
use crate::error::PaymentError;
use crate::model::{Allowance, TokenTransfer, Web3TransactionDao};
use crate::multi::check_allowance;
use crate::process::{process_transaction, ProcessTransactionResult};
use crate::transaction::{create_erc20_approve, create_erc20_transfer, create_eth_transfer};
use crate::utils::ConversionError;
use secp256k1::SecretKey;
use sqlx::{Connection, SqliteConnection};

use crate::setup::PaymentSetup;
use web3::types::{Address, U256};

#[derive(Eq, Hash, PartialEq, Debug, Clone)]
pub struct TokenTransferKey {
    pub from_addr: String,
    pub receiver_addr: String,
    pub chain_id: i64,
    pub token_addr: Option<String>,
}

pub async fn gather_transactions(
    conn: &mut SqliteConnection,
    payment_setup: &PaymentSetup,
) -> Result<u32, PaymentError> {
    let mut inserted_tx_count = 0;

    let mut hash_map = HashMap::<TokenTransferKey, Vec<TokenTransfer>>::new();

    let token_transfers = get_pending_token_transfers(conn).await?;

    for f in token_transfers.iter() {
        //group transactions
        let key = TokenTransferKey {
            from_addr: f.from_addr.clone(),
            receiver_addr: f.receiver_addr.clone(),
            chain_id: f.chain_id,
            token_addr: f.token_addr.clone(),
        };
        match hash_map.get_mut(&key) {
            Some(v) => {
                v.push(f.clone());
            }
            None => {
                hash_map.insert(key, vec![f.clone()]);
            }
        }
    }

    for pair in hash_map.iter_mut() {
        let token_transfers = pair.1;
        let token_transfer = pair.0;

        //sum of transfers

        let mut sum = U256::zero();
        for token_transfer in token_transfers.iter() {
            sum += U256::from_dec_str(&token_transfer.token_amount)?;
        }

        let chain_setup = payment_setup.get_chain_setup(token_transfer.chain_id)?;

        let max_fee_per_gas = chain_setup.max_fee_per_gas;
        let priority_fee = chain_setup.priority_fee;
        let web3 = payment_setup.get_provider(token_transfer.chain_id)?;

        log::debug!("Processing token transfer {:?}", token_transfer);
        let web3tx = if let Some(token_addr) = token_transfer.token_addr.as_ref() {
            //this is some arbitrary number.
            let minimum_allowance: U256 = U256::max_value() / U256::from(2);

            let mut db_allowance = find_allowance(
                conn,
                &token_transfer.from_addr,
                token_addr,
                &format!("{:#x}", *MULTI_ERC20_GOERLI),
                token_transfer.chain_id,
            )
            .await?;

            let allowance = match db_allowance.as_mut() {
                Some(db_allowance) => match db_allowance.confirm_date {
                    Some(_) => {
                        log::debug!("Allowance already confirmed from db");
                        U256::from_dec_str(&db_allowance.allowance)?
                    }
                    None => {
                        log::debug!("Allowance not confirmed in db, check on chain");
                        let allowance = check_allowance(
                            web3,
                            Address::from_str(&token_transfer.from_addr)?,
                            Address::from_str(token_addr)?,
                            *MULTI_ERC20_GOERLI,
                        )
                        .await?;
                        if allowance > minimum_allowance {
                            log::debug!("Allowance found on chain, update db");
                            db_allowance.confirm_date = Some(chrono::Utc::now());
                            update_allowance(conn, &db_allowance).await?;
                        }
                        allowance
                    }
                },
                None => {
                    log::debug!("No db entry, check allowance on chain");
                    let allowance = check_allowance(
                        web3,
                        Address::from_str(&token_transfer.from_addr)?,
                        Address::from_str(token_addr)?,
                        *MULTI_ERC20_GOERLI,
                    )
                    .await?;
                    if allowance > minimum_allowance {
                        log::debug!("Allowance found on chain, add entry to db");
                        let db_allowance = Allowance {
                            id: 0,
                            owner: token_transfer.from_addr.clone(),
                            token_addr: token_addr.clone(),
                            spender: format!("{:#x}", *MULTI_ERC20_GOERLI),
                            chain_id: token_transfer.chain_id,
                            tx_id: None,
                            allowance: allowance.to_string(),
                            confirm_date: Some(chrono::Utc::now()),
                            fee_paid: None,
                            error: None,
                        };
                        //allowance is confirmed on web3, update db
                        insert_allowance(conn, &db_allowance).await?;
                    }
                    allowance
                }
            };

            if allowance < minimum_allowance {
                let mut allowance = Allowance {
                    id: 0,
                    owner: token_transfer.from_addr.clone(),
                    token_addr: token_addr.clone(),
                    spender: format!("{:#x}", *MULTI_ERC20_GOERLI),
                    allowance: U256::max_value().to_string(),
                    chain_id: token_transfer.chain_id,
                    tx_id: None,
                    fee_paid: None,
                    confirm_date: None,
                    error: None,
                };

                let approve_tx = create_erc20_approve(
                    Address::from_str(&token_transfer.from_addr)?,
                    Address::from_str(&token_addr)?,
                    *MULTI_ERC20_GOERLI,
                    token_transfer.chain_id as u64,
                    1000,
                    max_fee_per_gas,
                    priority_fee,
                )?;
                let mut db_transaction = conn.begin().await?;
                let web3_tx_dao = insert_tx(&mut db_transaction, &approve_tx).await?;
                allowance.tx_id = Some(web3_tx_dao.id);
                insert_allowance(&mut db_transaction, &allowance).await?;

                db_transaction.commit().await?;
                inserted_tx_count += 1;

                return Ok(inserted_tx_count);
            }

            create_erc20_transfer(
                Address::from_str(&token_transfer.from_addr)?,
                Address::from_str(token_addr)?,
                Address::from_str(&token_transfer.receiver_addr)?,
                sum,
                token_transfer.chain_id as u64,
                1000,
                max_fee_per_gas,
                priority_fee,
            )?
        } else {
            create_eth_transfer(
                Address::from_str(&token_transfer.from_addr)?,
                Address::from_str(&token_transfer.receiver_addr)?,
                token_transfer.chain_id as u64,
                1000,
                max_fee_per_gas,
                priority_fee,
                sum,
            )
        };
        let mut db_transaction = conn.begin().await?;
        let web3_tx_dao = insert_tx(&mut db_transaction, &web3tx).await?;
        for token_transfer in token_transfers {
            token_transfer.tx_id = Some(web3_tx_dao.id);
            update_token_transfer(&mut db_transaction, &token_transfer).await?;
        }
        db_transaction.commit().await?;
        inserted_tx_count += 1;
    }

    Ok(inserted_tx_count)
}

pub async fn update_token_transfer_result(
    conn: &mut SqliteConnection,
    tx: &mut Web3TransactionDao,
    process_t_res: ProcessTransactionResult,
) -> Result<(), PaymentError> {
    match process_t_res {
        ProcessTransactionResult::Confirmed => {
            tx.processing = 0;

            let mut db_transaction = conn.begin().await?;
            let token_transfers = get_token_transfers_by_tx(&mut db_transaction, tx.id).await?;
            let token_transfers_count = U256::from(token_transfers.len() as u64);
            for mut token_transfer in token_transfers {
                if let Some(fee_paid) = tx.fee_paid.clone() {
                    let val = U256::from_dec_str(&fee_paid)
                        .map_err(|_err| ConversionError::from("failed to parse fee paid".into()))?;
                    let val2 = val / token_transfers_count;
                    token_transfer.fee_paid = Some(val2.to_string());
                } else {
                    token_transfer.fee_paid = None;
                }
                update_token_transfer(&mut db_transaction, &token_transfer).await?;
            }
            update_tx(&mut db_transaction, tx).await?;
            db_transaction.commit().await?;
        }
        ProcessTransactionResult::NeedRetry(err) => {
            tx.processing = 0;

            let mut db_transaction = conn.begin().await?;
            let token_transfers = get_token_transfers_by_tx(&mut db_transaction, tx.id).await?;
            for mut token_transfer in token_transfers {
                token_transfer.fee_paid = Some("0".to_string());
                token_transfer.error = Some(err.clone());
                update_token_transfer(&mut db_transaction, &token_transfer).await?;
            }
            update_tx(&mut db_transaction, tx).await?;
            db_transaction.commit().await?;
        },
        ProcessTransactionResult::InternalError(err) => {
            tx.processing = 0;

            let mut db_transaction = conn.begin().await?;
            let token_transfers = get_token_transfers_by_tx(&mut db_transaction, tx.id).await?;
            for mut token_transfer in token_transfers {
                token_transfer.fee_paid = Some("0".to_string());
                token_transfer.error = Some(err.clone());
                update_token_transfer(&mut db_transaction, &token_transfer).await?;
            }
            update_tx(&mut db_transaction, tx).await?;
            db_transaction.commit().await?;
        },
        ProcessTransactionResult::Unknown => {
            tx.processing = 1;
            update_tx(conn, tx).await?;
        }
    }
    Ok(())
}

pub async fn update_approve_result(
    conn: &mut SqliteConnection,
    tx: &mut Web3TransactionDao,
    process_t_res: ProcessTransactionResult,
) -> Result<(), PaymentError> {
    match process_t_res {
        ProcessTransactionResult::Confirmed => {
            tx.processing = 0;

            let mut db_transaction = conn.begin().await?;
            let mut allowance = get_allowance_by_tx(&mut db_transaction, tx.id).await?;
            allowance.fee_paid = tx.fee_paid.clone();
            update_allowance(&mut db_transaction, &allowance).await?;
            update_tx(&mut db_transaction, tx).await?;
            db_transaction.commit().await?;
        }
        ProcessTransactionResult::NeedRetry(err) => {
            tx.processing = 0;
            let mut db_transaction = conn.begin().await?;
            let mut allowance = get_allowance_by_tx(&mut db_transaction, tx.id).await?;
            allowance.fee_paid = Some("0".to_string());
            allowance.error = Some(err.clone());
            update_allowance(&mut db_transaction, &allowance).await?;
            update_tx(&mut db_transaction, tx).await?;
            db_transaction.commit().await?;
        }
        ProcessTransactionResult::InternalError(err) => {
            tx.processing = 0;
            let mut db_transaction = conn.begin().await?;
            let mut allowance = get_allowance_by_tx(&mut db_transaction, tx.id).await?;
            allowance.fee_paid = Some("0".to_string());
            allowance.error = Some(err.clone());
            update_allowance(&mut db_transaction, &allowance).await?;
            update_tx(&mut db_transaction, tx).await?;
            db_transaction.commit().await?;
        }
        ProcessTransactionResult::Unknown => {
            tx.processing = 1;
            update_tx(conn, tx).await?;
        }
    }
    Ok(())
}

pub async fn process_transactions(
    conn: &mut SqliteConnection,
    payment_setup: &PaymentSetup,
    secret_key: &SecretKey,
) -> Result<(), PaymentError> {
    loop {
        let mut transactions = get_transactions_being_processed(conn).await?;

        for tx in &mut transactions {
            let process_t_res =
                process_transaction(conn, tx, payment_setup, secret_key, false).await?;
            if tx.method == "ERC20.transfer" || tx.method == "transfer" {
                update_token_transfer_result(conn, tx, process_t_res).await?;
            } else if tx.method == "ERC20.approve" {
                update_approve_result(conn, tx, process_t_res).await?;
            }
            //process only one transaction at once
            break;
        }
        if transactions.is_empty() {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
    }
    Ok(())
}

pub async fn service_loop(
    conn: &mut SqliteConnection,
    payment_setup: PaymentSetup,
    secret_key: &SecretKey,
    finish_when_processed: bool,
) {
    let process_transactions_interval = 5;
    let gather_transactions_interval = 20;
    let mut last_update_time1 =
        chrono::Utc::now() - chrono::Duration::seconds(process_transactions_interval);
    let mut last_update_time2 =
        chrono::Utc::now() - chrono::Duration::seconds(gather_transactions_interval);

    let mut process_tx_needed = true;
    loop {
        let current_time = chrono::Utc::now();
        if current_time < last_update_time1 {
            //handle case when system time changed
            last_update_time1 = current_time;
        }

        if process_tx_needed
            && current_time
                > last_update_time1 + chrono::Duration::seconds(process_transactions_interval)
        {
            log::debug!("Processing transactions...");
            match process_transactions(conn, &payment_setup, secret_key).await {
                Ok(_) => {
                    //all pending transactions processed
                    process_tx_needed = false;
                }
                Err(e) => {
                    log::error!("Error in process transactions: {}", e);
                }
            };
            last_update_time1 = current_time;
        }

        if current_time
            > last_update_time2 + chrono::Duration::seconds(gather_transactions_interval)
        {
            log::debug!("Gathering transfers...");
            match gather_transactions(conn, &payment_setup).await {
                Ok(count) => {
                    if count > 0 {
                        process_tx_needed = true;
                    }
                }
                Err(e) => {
                    //if error happened, we should check if partial transfers were inserted
                    process_tx_needed = true;
                    log::error!("Error in gather transactions: {}", e);
                }
            };
            last_update_time2 = current_time;
            if finish_when_processed && !process_tx_needed {
                break;
            }
        }

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}
