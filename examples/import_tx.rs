use erc20_payment_lib::db::create_sqlite_connection;
use erc20_payment_lib::{config, err_custom_create};

use erc20_payment_lib::error::PaymentError;

use erc20_payment_lib::error::{CustomError, ErrorBag};
use erc20_payment_lib::misc::{display_private_keys, load_private_keys};
use std::env;
use std::str::FromStr;

use erc20_payment_lib::service::transaction_from_chain;
use erc20_payment_lib::setup::PaymentSetup;
use erc20_payment_lib::transaction::import_erc20_txs;
use structopt::StructOpt;
use web3::ethabi::ethereum_types::Address;

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
    let conn = create_sqlite_connection(Some(&db_conn), true).await?;

    let payment_setup =
        PaymentSetup::new(&config, vec![], vec![], true, false, false, 1, 1, false)?;
    let ps = payment_setup.chain_setup.get(&cli.chain_id).unwrap();
    let txs = import_erc20_txs(
        &ps.providers[0].provider,
        ps.glm_address.unwrap(),
        cli.chain_id,
        &[Address::from_str("0x0000000600000006000000060000000600000006").unwrap()],
    )
    .await
    .unwrap();

    for tx in &txs {
        transaction_from_chain(
            &ps.providers[0].provider,
            &conn,
            cli.chain_id,
            &format!("{tx:#x}"),
        )
        .await
        .unwrap();
    }

    conn.close().await; //it is needed to process all the transactions before closing the connection
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), PaymentError> {
    match main_internal().await {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("Error: {e}");
            Err(e)
        }
    }
}
