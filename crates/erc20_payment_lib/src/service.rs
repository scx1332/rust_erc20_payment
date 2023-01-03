
use std::collections::{BTreeMap, HashMap};
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::db::model::*;
use crate::db::ops::*;
use crate::error::{AllowanceRequest, ErrorBag, PaymentError};
use crate::multi::check_allowance;
use crate::process::{process_transaction, ProcessTransactionResult};
use crate::transaction::{
    create_erc20_approve, create_erc20_transfer, create_erc20_transfer_multi, create_eth_transfer,
    find_receipt_extended,
};
use crate::utils::ConversionError;

use crate::error::CustomError;
use crate::setup::{ChainSetup, PaymentSetup};
use crate::{err_create, err_custom_create, err_from};

use crate::runtime::SharedState;
use sqlx::{Connection, SqliteConnection};
use web3::transports::Http;
use web3::types::{Address, U256};
use web3::Web3;



pub async fn add_payment_request_2(
    conn: &mut SqliteConnection,
    token_address: Option<Address>,
    token_amount: U256,
    payment_id: &str,
    payer_addr: Address,
    receiver_addr: Address,
    chain_id: i64,
) -> Result<TransferInDao, PaymentError> {
    let transfer_in = TransferInDao {
        id: 0,
        payment_id: payment_id.to_string(),
        from_addr: format!("{:#x}", payer_addr),
        receiver_addr: format!("{:#x}", receiver_addr),
        chain_id,
        token_addr: token_address.map(|a| format!("{:#x}", a)),
        token_amount: token_amount.to_string(),
        tx_hash: None,
        requested_date: chrono::Utc::now(),
        received_date: None,
    };
    insert_transfer_in(conn, &transfer_in)
        .await
        .map_err(err_from!())
}

pub async fn add_glm_request(
    conn: &mut SqliteConnection,
    chain_setup: &ChainSetup,
    token_amount: U256,
    payment_id: &str,
    payer_addr: Address,
    receiver_addr: Address,
) -> Result<TransferInDao, PaymentError> {
    let transfer_in = TransferInDao {
        id: 0,
        payment_id: payment_id.to_string(),
        from_addr: format!("{:#x}", payer_addr),
        receiver_addr: format!("{:#x}", receiver_addr),
        chain_id: chain_setup.chain_id,
        token_addr: Some(format!(
            "{:#x}",
            chain_setup.glm_address.ok_or(err_custom_create!(
                "GLM address not set for chain {}",
                chain_setup.chain_id
            ))?
        )),
        token_amount: token_amount.to_string(),
        tx_hash: None,
        requested_date: chrono::Utc::now(),
        received_date: None,
    };
    insert_transfer_in(conn, &transfer_in)
        .await
        .map_err(err_from!())
}

pub async fn transaction_from_chain(
    web3: &Web3<Http>,
    conn: &mut SqliteConnection,
    chain_id: i64,
    tx_hash: &str,
) -> Result<bool, PaymentError> {
    println!("tx_hash: {}", tx_hash);
    let tx_hash = web3::types::H256::from_str(tx_hash)
        .map_err(|_err| ConversionError::from("Cannot parse tx_hash".to_string()))
        .map_err(err_from!())?;

    let (chain_tx_dao, transfers) = find_receipt_extended(web3, tx_hash, chain_id).await?;

    if chain_tx_dao.chain_status == 1 {
        let mut db_transaction = conn.begin().await.map_err(err_from!())?;

        let tx = insert_chain_tx(&mut db_transaction, &chain_tx_dao)
            .await
            .map_err(err_from!())?;
        for mut transfer in transfers {
            transfer.chain_tx_id = tx.id;
            insert_chain_transfer(&mut db_transaction, &transfer)
                .await
                .map_err(err_from!())?;
        }
        db_transaction.commit().await.map_err(err_from!())?;
        log::info!("Transaction found and parsed successfully: {}", tx.id);
    }

    Ok(true)
}

pub async fn confirm_loop(
    shared_state: Arc<Mutex<SharedState>>,
    conn: &mut SqliteConnection,
    payment_setup: &PaymentSetup,
) {
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(payment_setup.service_sleep)).await;
    }
}
