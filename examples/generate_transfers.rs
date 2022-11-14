use rust_erc20_payment::config;
use rust_erc20_payment::db::create_sqlite_connection;

use rust_erc20_payment::error::PaymentError;
use rust_erc20_payment::eth::get_eth_addr_from_secret;

use secp256k1::SecretKey;

use std::env;
use std::str::FromStr;
use structopt::StructOpt;
use rust_erc20_payment::misc::{create_test_amount_pool, generate_transaction_batch, null_address_pool, ordered_address_pool};

#[derive(Debug, StructOpt)]
struct TestOptions {
    #[structopt(long = "chain-name", default_value = "mumbai")]
    chain_name: String,

    #[structopt(long = "generate-count", default_value = "10")]
    generate_count: i64,
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

    let cli = TestOptions::from_args();

    let config = config::Config::load("config-payments.toml")?;
    let private_key = env::var("ETH_PRIVATE_KEY").unwrap();
    let secret_key = SecretKey::from_str(&private_key).unwrap();
    let public_addr = get_eth_addr_from_secret(&secret_key);

    let db_conn = env::var("DB_SQLITE_FILENAME").unwrap();
    let mut conn = create_sqlite_connection(&db_conn, true).await?;

    let addr_pool = ordered_address_pool(200000)?;
    let amount_pool = create_test_amount_pool(200000)?;

    let c = config.chain.get(&cli.chain_name).unwrap();
    generate_transaction_batch(
        &mut conn,
        c.network_id as u64,
        public_addr,
        Some(c.token.clone().unwrap().address),
        addr_pool,
        amount_pool,
        cli.generate_count as usize,
    )
    .await?;

    Ok(())
}
