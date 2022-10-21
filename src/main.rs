mod contracts;
mod model;
mod transaction;

use secp256k1::{PublicKey, SecretKey};
use std::fmt::Display;
use std::ops::Add;
use std::str::FromStr;
use std::{env, error, fmt};

use crate::contracts::{contract_encode, prepare_contract_template, ERC20_CONTRACT_TEMPLATE};
use crate::model::Web3TransactionDao;
use crate::transaction::{check_transaction, send_transaction};
use sha3::{Digest, Keccak256};
use web3::contract::tokens::Tokenize;
use web3::contract::Contract;
use web3::transports::Http;
use web3::types::{CallRequest, U64};
use web3::{
    ethabi::ethereum_types::U256,
    types::{Address, TransactionParameters},
    Error, Transport, Web3,
};

type Result2<T> = std::result::Result<T, Box<dyn error::Error>>;

/*
struct ERC20Payment {
    from: Address,
    to: Address,
    token: Address,
    amount: U256,
}
*/

struct Web3ChainConfig {
    glm_token: Address,
    chain_id: u64,
    erc20_contract: Contract<Http>,
}

pub async fn get_transaction_count(
    address: Address,
    web3: &Web3<Http>,
    pending: bool,
) -> Result2<u64> {
    let nonce_type = match pending {
        true => web3::types::BlockNumber::Pending,
        false => web3::types::BlockNumber::Latest,
    };
    let nonce = web3
        .eth()
        .transaction_count(address, Some(nonce_type))
        .await?;
    Ok(nonce.as_u64())
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
async fn main() -> web3::Result {
    let encoded_balance_of = contract_encode(
        &ERC20_CONTRACT_TEMPLATE,
        "balance_of",
        ("0x0000000000000000000000000000000000000000".to_string(),),
    );

    let prov_url = env::var("PROVIDER_URL").unwrap();
    let transport = web3::transports::Http::new(&prov_url)?;
    let web3 = web3::Web3::new(transport);

    let chain_id = web3.eth().chain_id().await?.as_u64();
    // Insert the 20-byte "to" address in hex format (prefix with 0x)
    let to = Address::from_str("0xc596aee002ebe98345ce3f967631aaf79cfbdf41").unwrap();

    // Insert the 32-byte private key in hex format (do NOT prefix with 0x)
    let private_key = env::var("ETH_PRIVATE_KEY").unwrap();

    let secret_key = SecretKey::from_str(&private_key).unwrap();
    let from_addr = get_eth_addr_from_secret(&secret_key);

    let nonce = get_transaction_count(from_addr, &web3, false)
        .await
        .unwrap();
    println!("nonce: {}", nonce);

    let priority_fee = if chain_id == 5 {
        1_100_000_000_u64
    } else {
        panic!("Chain ID not supported");
    };

    let mut web3_tx_dao = Web3TransactionDao {
        from: format!("{from_addr:#x}"),
        to: format!("{to:#x}"),
        chain_id: chain_id,
        gas_limit: 1000,
        total_fee: U256::from(1000_000_000_000_u64).to_string(),
        priority_fee: priority_fee.to_string(),
        value: "1000000000000000000000000000".to_string(),
        nonce: nonce,
        data: None,
        signed_raw_data: None,
    };

    println!("web3_tx_dao: {:?}", web3_tx_dao);

    check_transaction(&web3, &mut web3_tx_dao).await?;

    send_transaction(&web3, &mut web3_tx_dao).await?;

    //println!("Transaction hash: {:?}", signed.transaction_hash);
    //println!("Transaction payload: {:?}", signed.raw_transaction);

    //println!("Tx succeeded with hash: {:#x}", result);

    Ok(())
}
