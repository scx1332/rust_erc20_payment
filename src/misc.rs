use crate::db::operations::insert_token_transfer;
use std::str::FromStr;

use crate::transaction::create_token_transfer;

use sqlx::SqliteConnection;

use crate::error::PaymentError;
use rand::Rng;
use web3::types::{Address, U256};
use crate::err_from;
use crate::error::*;

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
        addr_pool.push(Address::from_str(&format!(
            "0x{0:0>8}{0:0>8}{0:0>8}{0:0>8}{0:0>8}",
            i
        )).map_err(err_from!())?);
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

#[allow(unused)]
pub async fn generate_transaction_batch(
    conn: &mut SqliteConnection,
    chain_id: u64,
    from: Address,
    token_addr: Option<Address>,
    addr_pool: Vec<Address>,
    amount_pool: Vec<U256>,
    number_of_transfers: usize,
) -> Result<(), PaymentError> {
    //thread rng
    let mut rng = rand::thread_rng();
    for transaction_no in 0..number_of_transfers {
        let receiver = addr_pool[rng.gen_range(0..addr_pool.len())];
        let amount = amount_pool[rng.gen_range(0..amount_pool.len())];
        let token_transfer = create_token_transfer(from, receiver, chain_id, token_addr, amount);
        let _token_transfer = insert_token_transfer(conn, &token_transfer).await.map_err(err_from!())?;
        log::info!(
            "Generated token transfer: {:?} {}/{}",
            token_transfer,
            transaction_no,
            number_of_transfers
        );
    }
    Ok(())
}
