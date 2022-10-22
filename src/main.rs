mod contracts;
mod model;
mod transaction;
mod utils;
mod process;

use secp256k1::{PublicKey, SecretKey};

use std::str::FromStr;
use std::time::Duration;
use std::{env, error, fmt};

use crate::contracts::{contract_encode, ERC20_CONTRACT_TEMPLATE};
use crate::model::Web3TransactionDao;
use crate::transaction::{check_transaction, create_eth_transfer, find_receipt, find_tx, send_transaction, sign_transaction};
use sha3::{Digest, Keccak256};

use web3::contract::Contract;
use web3::transports::Http;

use crate::utils::gwei_to_u256;
use web3::{ethabi::ethereum_types::U256, types::Address, Web3};
use crate::process::process_transaction;

type Result2<T> = std::result::Result<T, Box<dyn error::Error>>;

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
async fn main() -> Result<(), Box<dyn error::Error>> {
    let _encoded_balance_of = contract_encode(
        &ERC20_CONTRACT_TEMPLATE,
        "balance_of",
        ("0x0000000000000000000000000000000000000000".to_string(),),
    );

    let prov_url = env::var("PROVIDER_URL").unwrap();
    let transport = web3::transports::Http::new(&prov_url)?;
    let web3 = web3::Web3::new(transport);

    let chain_id = web3.eth().chain_id().await?.as_u64();
    // Insert the 20-byte "to" address in hex format (prefix with 0x)

    // Insert the 32-byte private key in hex format (do NOT prefix with 0x)
    let private_key = env::var("ETH_PRIVATE_KEY").unwrap();

    let secret_key = SecretKey::from_str(&private_key).unwrap();
    let from_addr = get_eth_addr_from_secret(&secret_key);
    let to = from_addr;

    let (total_fee, priority_fee) = if chain_id == 5 {
        (gwei_to_u256(1000.0)?, gwei_to_u256(1.111)?)
    } else {
        panic!("Chain ID not supported");
    };

    let mut web3_tx_dao = create_eth_transfer(
        from_addr,
        to,
        chain_id,
        0,
        total_fee,
        priority_fee,
        U256::from(1),
    );
    let mut web3_tx_dao2 = web3_tx_dao.clone();
    let process_t_res = process_transaction(&mut web3_tx_dao, &web3, &secret_key);
    let process_t_res2 = process_transaction(&mut web3_tx_dao2, &web3, &secret_key);

    let (res1, res2) = tokio::join!(process_t_res, process_t_res2);
    println!("Transaction 1: {:?}", res1?);
    println!("Transaction 2: {:?}", res2?);

    //println!("Transaction hash: {:?}", signed.transaction_hash);
    //println!("Transaction payload: {:?}", signed.raw_transaction);

    //println!("Tx succeeded with hash: {:#x}", result);

    Ok(())
}
