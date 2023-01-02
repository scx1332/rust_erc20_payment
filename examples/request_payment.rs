use erc20_payment_lib::db::create_sqlite_connection;
use erc20_payment_lib::{config, err_custom_create, err_from};

use erc20_payment_lib::error::PaymentError;

use erc20_payment_lib::error::{CustomError, ErrorBag};
use erc20_payment_lib::misc::{display_private_keys, load_private_keys};
use sqlx::Connection;
use std::env;
use std::str::FromStr;

use erc20_payment_lib::service::add_payment_request;
use erc20_payment_lib::setup::PaymentSetup;

use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use structopt::StructOpt;
use web3::ethabi::ethereum_types::Address;
use web3::types::U256;

fn random_string(n: usize) -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(n)
        .map(char::from)
        .collect()
}

#[derive(Debug, StructOpt)]
struct ImportTxOptions {
    #[structopt(long = "chain-id", default_value = "987789")]
    chain_id: i64,

    #[structopt(
        long = "tx-hash",
        default_value = "0x13d8a54dec1c0a30f1cd5129f690c3e27b9aadd59504957bad4d247966dadae7"
    )]
    _tx_hash: String,
}

async fn main_internal() -> Result<(), PaymentError> {
    if let Err(err) = dotenv::dotenv() {
        return Err(err_custom_create!("No .env file found: {}", err));
    }
    env_logger::init();

    let cli: ImportTxOptions = ImportTxOptions::from_args();

    let config = config::Config::load("config-payments.toml")?;

    let (private_keys, _public_addrs) = load_private_keys(&env::var("ETH_PRIVATE_KEYS").unwrap())?;
    display_private_keys(&private_keys);

    let db_conn = env::var("DB_SQLITE_FILENAME").unwrap();
    let mut conn = create_sqlite_connection(&db_conn, true).await?;

    let payment_setup = PaymentSetup::new(&config, vec![], true, false, false, 1, 1, false)?;
    let chain_setup = payment_setup.chain_setup.get(&cli.chain_id).unwrap();

    let payment_request = add_payment_request(
        &mut conn,
        chain_setup,
        U256::from(1000),
        &uuid::Uuid::new_v4().to_string(),
        Address::from_str("0x001066290077e38f222cc6009c0c7a91d5192303").unwrap(),
        Address::from_str("0x0000000600000006000000060000000600000006").unwrap(),
    )
    .await
    .unwrap();
    println!("Added payment_request: {:?}", payment_request);

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
