use rust_erc20_payment::config;
use rust_erc20_payment::db::create_sqlite_connection;
use rust_erc20_payment::db::operations::insert_token_transfer;
use rust_erc20_payment::error::PaymentError;
use rust_erc20_payment::eth::get_eth_addr_from_secret;
use rust_erc20_payment::transaction::create_token_transfer;
use secp256k1::SecretKey;
use sqlx::SqliteConnection;
use std::env;
use std::str::FromStr;
use rand::Rng;
use web3::types::{Address, U256};

fn create_test_address_pool() -> Result<Vec::<Address>, PaymentError>{
    let mut addr_pool = Vec::<Address>::new();
    for i in 0..2000 {
        addr_pool.push(Address::from_low_u64_le(i + 100));
    }
    Ok(addr_pool)
}

fn create_test_amount_pool() -> Result<Vec::<U256>, PaymentError>{
    let mut amount_pool = Vec::<U256>::new();
    for i in 0..2000 {
        amount_pool.push(U256::from(i + 100));
    }
    Ok(amount_pool)
}

async fn generate_transaction_batch(
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
        let _token_transfer = insert_token_transfer(conn, &token_transfer).await?;
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), PaymentError> {
    if let Err(err) = dotenv::dotenv() {
        return Err(PaymentError::OtherError(format!(
            "No .env file found: {}",
            err
        )));
    }
    env_logger::init();

    let config = config::Config::load("config-payments.toml")?;
    let private_key = env::var("ETH_PRIVATE_KEY").unwrap();
    let secret_key = SecretKey::from_str(&private_key).unwrap();
    let public_addr = get_eth_addr_from_secret(&secret_key);

    let mut conn = create_sqlite_connection("db.sqlite", true).await?;


    let addr_pool = create_test_address_pool()?;
    let amount_pool = create_test_amount_pool()?;

    let c = config.chain.get("mumbai").unwrap();
    generate_transaction_batch(
        &mut conn,
        c.network_id as u64,
        public_addr,
        Some(c.token.clone().unwrap().address),
        addr_pool,
        amount_pool,
        10,
    )
    .await?;

    Ok(())
}
