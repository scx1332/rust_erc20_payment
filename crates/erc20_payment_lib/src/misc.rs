use crate::db::ops::insert_token_transfer;
use std::str::FromStr;
use std::time::Instant;

use crate::transaction::create_token_transfer;

use sqlx::SqlitePool;

use crate::error::PaymentError;
use crate::error::*;
use crate::eth::get_eth_addr_from_secret;
use crate::service::add_payment_request_2;
use crate::{err_custom_create, err_from};
use rand::Rng;
use secp256k1::SecretKey;

use web3::types::{Address, U256};

#[allow(unused)]
pub fn null_address_pool() -> Result<Vec<Address>, PaymentError> {
    ordered_address_pool(1, true)
}

#[allow(unused)]
pub fn ordered_address_pool(
    size: usize,
    include_null_address: bool,
) -> Result<Vec<Address>, PaymentError> {
    let mut addr_pool = Vec::<Address>::new();
    let range = if include_null_address {
        0..size
    } else {
        1..(size + 1)
    };
    for i in range {
        //if i equals to 0 then null address is generated
        addr_pool.push(
            Address::from_str(&format!("0x{i:0>8}{i:0>8}{i:0>8}{i:0>8}{i:0>8}"))
                .map_err(err_from!())?,
        );
    }
    Ok(addr_pool)
}

#[allow(unused)]
pub fn create_test_amount_pool(size: usize) -> Result<Vec<U256>, PaymentError> {
    let mut amount_pool = Vec::<U256>::new();
    for i in 0..size {
        amount_pool.push(U256::from(i + 100));
    }
    Ok(amount_pool)
}

pub async fn generate_transaction_batch(
    conn: &SqlitePool,
    chain_id: i64,
    from_addr_pool: &[Address],
    token_addr: Option<Address>,
    addr_pool: &[Address],
    amount_pool: &[U256],
    number_of_transfers: usize,
) -> Result<(), PaymentError> {
    //thread rng
    let mut rng = rand::thread_rng();
    let max_block_db_interval = std::time::Duration::from_secs(1);
    let release_db_interval = std::time::Duration::from_secs(1);

    let mut last_time = Instant::now();
    for transaction_no in 0..number_of_transfers {
        let receiver = addr_pool[rng.gen_range(0..addr_pool.len())];
        let amount = amount_pool[rng.gen_range(0..amount_pool.len())];
        let from = from_addr_pool[rng.gen_range(0..from_addr_pool.len())];
        let payment_id = uuid::Uuid::new_v4().to_string();
        let token_transfer = create_token_transfer(
            from,
            receiver,
            chain_id,
            Some(&payment_id),
            token_addr,
            amount,
        );
        let _token_transfer = insert_token_transfer(conn, &token_transfer)
            .await
            .map_err(err_from!())?;
        add_payment_request_2(
            conn,
            token_addr,
            amount,
            &payment_id,
            from,
            receiver,
            chain_id,
        )
        .await?;
        log::info!(
            "Generated token transfer: from: {} to: {} {}/{}",
            token_transfer.from_addr,
            token_transfer.receiver_addr,
            transaction_no,
            number_of_transfers
        );
        let curr_time = Instant::now();
        if curr_time - last_time > max_block_db_interval {
            log::info!("Waiting for db to release");
            tokio::time::sleep(release_db_interval).await;
            last_time = Instant::now();
        }
    }
    Ok(())
}

pub fn load_private_keys(str: &str) -> Result<(Vec<SecretKey>, Vec<Address>), PaymentError> {
    let mut keys = Vec::new();
    let mut addrs = Vec::new();
    if str.is_empty() {
        return Ok((keys, addrs));
    }
    for key in str.split(',') {
        //do not disclose the private key in error message
        let secret = SecretKey::from_str(key)
            .map_err(|_| err_custom_create!("Failed to parse private key"))?;
        let public_addr = get_eth_addr_from_secret(&secret);
        keys.push(secret);
        addrs.push(public_addr);
    }
    Ok((keys, addrs))
}

pub fn load_public_addresses(str: &str) -> Result<Vec<Address>, PaymentError> {
    let mut addrs = Vec::new();
    if str.is_empty() {
        return Ok(addrs);
    }
    for key in str.split(',') {
        let addr = Address::from_str(key).map_err(err_from!())?;
        addrs.push(addr);
    }
    Ok(addrs)
}

pub fn display_private_keys(keys: &[SecretKey]) {
    let mut account_no = 1;
    if keys.is_empty() {
        log::info!("No Eth accounts loaded");
    } else {
        for key in keys {
            let public_addr = get_eth_addr_from_secret(key);
            if keys.len() >= 10 {
                log::info!("Eth account loaded {:02}: {:?}", account_no, public_addr);
            } else {
                log::info!("Eth account loaded {}: {:?}", account_no, public_addr);
            }
            account_no += 1;
        }
    }
}
