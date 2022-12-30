use erc20_payment_lib::db::create_sqlite_connection;
use erc20_payment_lib::{config, err_custom_create, err_from};

use erc20_payment_lib::error::PaymentError;

use erc20_payment_lib::error::{CustomError, ErrorBag};
use erc20_payment_lib::misc::{
    create_test_amount_pool, display_private_keys, generate_transaction_batch, load_private_keys,
    ordered_address_pool,
};
use sqlx::Connection;
use std::env;

use erc20_payment_lib::service::transaction_from_chain;
use erc20_payment_lib::setup::PaymentSetup;
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

    let (private_keys, public_addrs) = load_private_keys(&env::var("ETH_PRIVATE_KEYS").unwrap())?;
    display_private_keys(&private_keys);

    let db_conn = env::var("DB_SQLITE_FILENAME").unwrap();
    let mut conn = create_sqlite_connection(&db_conn, true).await?;

    //let web3 = web3::Web3::new(config.chain.get("dev").unwrap().rpc_endpoints[0]);
    let payment_setup = PaymentSetup::new(&config, vec![], true, false, false, 1, 1, false)?;
    let ps = payment_setup.chain_setup.get(&987789).unwrap();
    transaction_from_chain(
        &ps.providers[0].provider,
        &mut conn,
        987789,
        "0x13d8a54dec1c0a30f1cd5129f690c3e27b9aadd59504957bad4d247966dadae7",
    )
    .await
    .unwrap();

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
