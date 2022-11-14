use crate::db::operations::insert_token_transfer;

use crate::transaction::create_token_transfer;

use sqlx::SqliteConnection;

use crate::error::PaymentError;
use rand::Rng;
use web3::types::{Address, U256};

pub fn create_test_address_pool() -> Result<Vec<Address>, PaymentError> {
    let mut addr_pool = Vec::<Address>::new();
    for i in 0..2000 {
        addr_pool.push(Address::from_low_u64_le(i + 100));
    }
    Ok(addr_pool)
}

pub fn create_test_amount_pool() -> Result<Vec<U256>, PaymentError> {
    let mut amount_pool = Vec::<U256>::new();
    for i in 0..2000 {
        amount_pool.push(U256::from(i + 100));
    }
    Ok(amount_pool)
}

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
    for _transaction_no in 0..number_of_transfers {
        let receiver = addr_pool[rng.gen_range(0..addr_pool.len())];
        let amount = amount_pool[rng.gen_range(0..amount_pool.len())];
        let token_transfer = create_token_transfer(from, receiver, chain_id, token_addr, amount);
        let _token_transfer = insert_token_transfer(conn, &token_transfer).await?;
        log::info!("Generated token transfer: {:?}", token_transfer);
    }
    Ok(())
}
