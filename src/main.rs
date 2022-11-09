mod config;
mod contracts;
mod db;
mod error;
mod eth;
mod model;
mod multi;
mod options;
mod process;
mod service;
mod setup;
mod transaction;
mod utils;

use secp256k1::{PublicKey, SecretKey};

use std::str::FromStr;

use std::time::Duration;
use std::{env, fmt};

use crate::transaction::create_token_transfer;
use sha3::{Digest, Keccak256};

use web3::contract::Contract;
use web3::transports::Http;

use web3::types::Address;

use crate::db::create_sqlite_connection;
use crate::db::operations::insert_token_transfer;
use crate::error::PaymentError;

use crate::options::validated_cli;
use crate::service::service_loop;
use crate::setup::PaymentSetup;

struct _Web3ChainConfig {
    glm_token: Address,
    chain_id: u64,
    erc20_contract: Contract<Http>,
}

pub fn get_eth_addr_from_secret(secret_key: &SecretKey) -> Address {
    Address::from_slice(
        &Keccak256::digest(
            &PublicKey::from_secret_key(&secp256k1::Secp256k1::new(), &secret_key)
                .serialize_uncompressed()[1..65],
        )
        .as_slice()[12..],
    )
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

/// Below sends a transaction to a local node that stores private keys (eg Ganache)
/// For generating and signing a transaction offline, before transmitting it to a public node (eg Infura) see transaction_public
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
    let payment_setup = PaymentSetup::new(&config)?;
    log::debug!("Payment setup: {:?}", payment_setup);

    let mut conn = create_sqlite_connection("db.sqlite", true).await?;

    let private_key = env::var("ETH_PRIVATE_KEY").unwrap();
    let secret_key = SecretKey::from_str(&private_key).unwrap();
    let from_addr = get_eth_addr_from_secret(&secret_key);

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
        let _token_transfer = insert_token_transfer(&mut conn, &token_transfer).await?;
    }

    //service_loop(&mut conn, &web3, &secret_key).await;
    tokio::spawn(async move {
        service_loop(&mut conn, payment_setup, &secret_key, !cli.keep_running).await
    });

    loop {
        //wait
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
