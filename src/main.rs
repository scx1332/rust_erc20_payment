mod contracts;
mod db;
mod error;
mod eth;
mod model;
mod multi;
mod options;
mod process;
mod service;
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

/*
struct ERC20Payment {
    from: Address,
    to: Address,
    token: Address,
    amount: U256,
}
*/

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

/*
fn prepare_erc20_contract(
    ethereum_client: &Web3<Http>,
    env: &config::EnvConfiguration,
) -> Result<Contract<Http>, GenericError> {
    prepare_contract(
        ethereum_client,
        env.glm_contract_address,
        include_bytes!("../contracts/ierc20.json"),
    )
}*/

/*
fn prepare_erc20_multi_contract(
    ethereum_client: &Web3<Http>,
    env: &config::EnvConfiguration,
) -> Result<Contract<Http>, GenericError> {
    prepare_contract(
        env.glm_multi_transfer_contract_address
            .ok_or(GenericError::new(
                "No multipayment contract defined for this environment",
            ))?,
        include_bytes!("contracts/multi_transfer_erc20.json"),
    )
}*/

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

    // let conn = SqliteConnectOptions::from_str("sqlite://db.sqlite")?.create_if_missing(true).connect().await?;

    let mut conn = create_sqlite_connection(2).await?;

    let prov_url = env::var("PROVIDER_URL").unwrap();
    let transport = web3::transports::Http::new(&prov_url)?;
    let web3 = web3::Web3::new(transport);

    let chain_id = web3.eth().chain_id().await?.as_u64();
    // Insert the 20-byte "to" address in hex format (prefix with 0x)

    // Insert the 32-byte private key in hex format (do NOT prefix with 0x)
    let private_key = env::var("ETH_PRIVATE_KEY").unwrap();

    let secret_key = SecretKey::from_str(&private_key).unwrap();
    let from_addr = get_eth_addr_from_secret(&secret_key);

    let _token_addr = if chain_id == 5 {
        Address::from_str("0x33af15c79d64b85ba14aaffaa4577949104b22e8").unwrap()
    } else if chain_id == 80001 {
        Address::from_str("0x2036807b0b3aaf5b1858ee822d0e111fddac7018").unwrap()
    } else {
        panic!("Chain ID not supported");
    };

    for transaction_no in 0..cli.receivers.len() {
        let receiver = cli.receivers[transaction_no];
        let amount = cli.amounts[transaction_no];
        let token_transfer =
            create_token_transfer(from_addr, receiver, cli.chain_id as u64, cli.token_addr, amount);
        let _token_transfer = insert_token_transfer(&mut conn, &token_transfer).await?;
    }

    //service_loop(&mut conn, &web3, &secret_key).await;
    tokio::spawn(async move { service_loop(&mut conn, &web3, &secret_key).await });

    loop {
        //wait
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
