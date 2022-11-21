mod config;
mod contracts;
mod db;
mod error;
mod eth;
mod misc;
mod model;
mod multi;
mod options;
mod process;
mod service;
mod setup;
mod transaction;
mod utils;

use secp256k1::SecretKey;

use std::str::FromStr;

use sqlx::SqliteConnection;
use std::{env, fmt};

use crate::transaction::create_token_transfer;

use web3::contract::Contract;
use web3::transports::Http;

use web3::types::Address;

use crate::db::create_sqlite_connection;
use crate::db::operations::insert_token_transfer;
use crate::error::PaymentError;
use crate::eth::get_eth_addr_from_secret;

use crate::options::{validated_cli, ValidatedOptions};
use crate::service::service_loop;
use crate::setup::PaymentSetup;

struct _Web3ChainConfig {
    glm_token: Address,
    chain_id: u64,
    erc20_contract: Contract<Http>,
}

struct HexSlice<'a>(&'a [u8]);

impl<'a> HexSlice<'a> {
    fn new<T>(data: &'a T) -> HexSlice<'a>
    where
        T: ?Sized + AsRef<[u8]> + 'a,
    {
        HexSlice(data.as_ref())
    }
}

// You can choose to implement multiple traits, like Lower and UpperHex
impl fmt::Display for HexSlice<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0 {
            // Decide if you want to pad the value or have spaces inbetween, etc.
            write!(f, "{:X} ", byte)?;
        }
        Ok(())
    }
}

trait HexDisplayExt {
    fn hex_display(&self) -> HexSlice<'_>;
}

impl<T> HexDisplayExt for T
where
    T: ?Sized + AsRef<[u8]>,
{
    fn hex_display(&self) -> HexSlice<'_> {
        HexSlice::new(self)
    }
}

async fn process_cli(
    conn: &mut SqliteConnection,
    cli: &ValidatedOptions,
    secret_key: &SecretKey,
) -> Result<(), PaymentError> {
    let from_addr = get_eth_addr_from_secret(secret_key);
    for transaction_no in 0..cli.receivers.len() {
        let receiver = cli.receivers[transaction_no];
        let amount = cli.amounts[transaction_no];
        let token_transfer = create_token_transfer(
            from_addr,
            receiver,
            cli.chain_id as u64,
            cli.token_addr,
            amount,
        );
        let _token_transfer = insert_token_transfer(conn, &token_transfer).await?;
    }
    Ok(())

    //service_loop(&mut conn, &web3, &secret_key).await;
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
    let cli = validated_cli()?;

    let config = config::Config::load("config-payments.toml")?;
    let private_key = env::var("ETH_PRIVATE_KEY").unwrap();
    let secret_key = SecretKey::from_str(&private_key).unwrap();
    let payment_setup = PaymentSetup::new(
        &config,
        secret_key,
        !cli.keep_running,
        cli.generate_tx_only,
        cli.skip_multi_contract_check,
    )?;
    log::debug!("Payment setup: {:?}", payment_setup);

    let db_conn = env::var("DB_SQLITE_FILENAME").unwrap();
    let mut conn = create_sqlite_connection(&db_conn, true).await?;

    process_cli(&mut conn, &cli, &payment_setup.secret_key).await?;

    let sp = tokio::spawn(async move { service_loop(&mut conn, payment_setup).await });
    sp.await
        .map_err(|e| PaymentError::OtherError(format!("Service loop failed: {:?}", e)))?;
    Ok(())
}
