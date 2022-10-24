mod contracts;
mod model;
mod process;
mod transaction;
mod utils;
mod eth;

use secp256k1::{PublicKey, SecretKey};

use std::str::FromStr;

use std::{env, error, fmt};

use crate::contracts::{contract_encode, ERC20_CONTRACT_TEMPLATE};
use crate::model::Web3TransactionDao;
use crate::transaction::{check_transaction, create_erc20_transfer, create_eth_transfer, find_receipt, send_transaction, sign_transaction};
use sha3::{Digest, Keccak256};

use web3::contract::Contract;
use web3::transports::Http;

use crate::process::process_transaction;
use crate::utils::gwei_to_u256;
use web3::{ethabi::ethereum_types::U256, types::Address, Web3};

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
    env_logger::init();

    let prov_url = env::var("PROVIDER_URL").unwrap();
    let transport = web3::transports::Http::new(&prov_url)?;
    let web3 = web3::Web3::new(transport);

    let chain_id = web3.eth().chain_id().await?.as_u64();
    // Insert the 20-byte "to" address in hex format (prefix with 0x)

    // Insert the 32-byte private key in hex format (do NOT prefix with 0x)
    let private_key = env::var("ETH_PRIVATE_KEY").unwrap();

    let secret_key = SecretKey::from_str(&private_key).unwrap();
    let from_addr = get_eth_addr_from_secret(&secret_key);
    let to = Address::from_str(&env::var("ETH_TO_ADDRESS").unwrap()).unwrap();

    let (max_fee_per_gas, priority_fee) = if chain_id == 5 {
        (gwei_to_u256(1000.0)?, gwei_to_u256(1.111)?)
    } else {
        panic!("Chain ID not supported");
    };

    /*
    let mut web3_tx_dao = create_eth_transfer(
        from_addr,
        to,
        chain_id,
        0,
        max_fee_per_gas,
        priority_fee,
        U256::from(1),
    );*/
    let mut web3_tx_dao = create_erc20_transfer(
        from_addr,
        Address::from_str(&env::var("ETH_TOKEN_ADDRESS").unwrap()).unwrap(),
        to,
        U256::from(1),
        chain_id,
        1000,
        max_fee_per_gas,
        priority_fee,
    )?;



    let mut web3_tx_dao2 = web3_tx_dao.clone();
    let process_t_res = process_transaction(&mut web3_tx_dao, &web3, &secret_key);
    //web3_tx_dao2.value = "2".to_string();
    let process_t_res2 = process_transaction(&mut web3_tx_dao2, &web3, &secret_key);

    let (res1, res2) = tokio::join!(process_t_res, process_t_res2);
    println!("Transaction 1: {:?}", res1?);
    println!("Transaction 2: {:?}", res2?);

    //println!("Transaction hash: {:?}", signed.transaction_hash);
    //println!("Transaction payload: {:?}", signed.raw_transaction);

    //println!("Tx succeeded with hash: {:#x}", result);

    Ok(())
}
