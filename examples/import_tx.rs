use erc20_payment_lib::db::create_sqlite_connection;
use erc20_payment_lib::{config, err_custom_create, err_from};

use erc20_payment_lib::error::PaymentError;

use erc20_payment_lib::error::{CustomError, ErrorBag};
use erc20_payment_lib::misc::{display_private_keys, load_private_keys};
use sqlx::Connection;
use std::env;

use erc20_payment_lib::service::transaction_from_chain;
use erc20_payment_lib::setup::PaymentSetup;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct ImportTxOptions {
    #[structopt(long = "chain-id", default_value = "987789")]
    chain_id: i64,

    #[structopt(
        long = "tx-hash",
        default_value = "0x13d8a54dec1c0a30f1cd5129f690c3e27b9aadd59504957bad4d247966dadae7"
    )]
    tx_hash: String,
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

    //let web3 = web3::Web3::new(config.chain.get("dev").unwrap().rpc_endpoints[0]);
    let payment_setup = PaymentSetup::new(&config, vec![], true, false, false, 1, 1, false)?;
    let ps = payment_setup.chain_setup.get(&cli.chain_id).unwrap();
    transaction_from_chain(
        &ps.providers[0].provider,
        &mut conn,
        cli.chain_id,
        &cli.tx_hash,
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
