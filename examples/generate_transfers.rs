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
use rust_erc20_payment::misc::{create_test_address_pool, create_test_amount_pool, generate_transaction_batch};

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

    let db_conn = env::var("DB_SQLITE_FILENAME").unwrap();
    let mut conn = create_sqlite_connection(&db_conn, true).await?;

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
