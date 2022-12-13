use erc20_payment_lib::db::create_sqlite_connection;
use erc20_payment_lib::{config, err_custom_create, err_from};

use erc20_payment_lib::error::PaymentError;
use erc20_payment_lib::eth::get_eth_addr_from_secret;

use secp256k1::SecretKey;

use erc20_payment_lib::error::{CustomError, ErrorBag};
use erc20_payment_lib::misc::{
    create_test_amount_pool, generate_transaction_batch, ordered_address_pool,
};
use sqlx::Connection;
use std::env;
use std::str::FromStr;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct TestOptions {
    #[structopt(long = "chain-name", default_value = "mumbai")]
    chain_name: String,

    #[structopt(long = "generate-count", default_value = "10")]
    generate_count: usize,

    #[structopt(long = "address-pool-size", default_value = "10")]
    address_pool_size: usize,

    #[structopt(long = "amounts-pool-size", default_value = "10")]
    amounts_pool_size: usize,
}

async fn main_internal() -> Result<(), PaymentError> {
    if let Err(err) = dotenv::dotenv() {
        return Err(err_custom_create!("No .env file found: {}", err));
    }
    env_logger::init();

    let cli: TestOptions = TestOptions::from_args();

    let config = config::Config::load("config-payments.toml")?;
    let private_key = env::var("ETH_PRIVATE_KEY").unwrap();
    let secret_key = SecretKey::from_str(&private_key).unwrap();
    let public_addr = get_eth_addr_from_secret(&secret_key);

    let db_conn = env::var("DB_SQLITE_FILENAME").unwrap();
    let mut conn = create_sqlite_connection(&db_conn, true).await?;

    let addr_pool = ordered_address_pool(cli.address_pool_size, false)?;
    let amount_pool = create_test_amount_pool(cli.amounts_pool_size)?;

    let c = config.chain.get(&cli.chain_name).unwrap();
    generate_transaction_batch(
        &mut conn,
        c.network_id as u64,
        public_addr,
        Some(c.token.clone().unwrap().address),
        addr_pool,
        amount_pool,
        cli.generate_count,
    )
    .await?;

    conn.close().await.map_err(err_from!())?; //it is needed to process all the transactions before closing the connection
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), PaymentError> {
    match main_internal().await {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("Error: {}", e);
            Err(e)
        }
    }
}
