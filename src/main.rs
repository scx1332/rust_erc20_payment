use secp256k1::{PublicKey, SecretKey};
use std::{env, fmt};
use std::ops::Add;
use std::str::FromStr;

use sha3::{Digest, Keccak256};
use web3::transports::Http;
use web3::types::{Transaction, U64};
use web3::{
    ethabi::ethereum_types::U256,
    types::{Address, TransactionParameters},
    Error, Web3,
};

/*
struct ERC20Payment {
    from: Address,
    to: Address,
    token: Address,
    amount: U256,
}
*/

pub async fn get_transaction_count(
    address: Address,
    web3: &Web3<Http>,
    pending: bool,
) -> Result<u64, Error> {
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

#[derive(Debug, Clone)]
struct Erc20TransactionDao {
    from: String,
    to: String,
    token: String,
    chain_id: u64,
    gas_limit: u64,
    max_fee_per_gas: String,
    priority_fee: String,
    amount: String,
    nonce: u64,

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

fn dao_to_transaction(erc20_tx_dao: Erc20TransactionDao) -> TransactionParameters {
    let from = Address::from_str(&erc20_tx_dao.from[2..]).unwrap();
    let to = Address::from_str(&erc20_tx_dao.to[2..]).unwrap();
    let token = Address::from_str(&erc20_tx_dao.token[2..]).unwrap();
    let chain_id = erc20_tx_dao.chain_id;
    let gas_limit = erc20_tx_dao.gas_limit;
    let total_fee = U256::from_dec_str(&erc20_tx_dao.total_fee).unwrap();
    let priority_fee = U256::from_dec_str(&erc20_tx_dao.priority_fee).unwrap();
    let amount = U256::from_dec_str(&erc20_tx_dao.amount).unwrap();
    let nonce = erc20_tx_dao.nonce;

    // Build the tx object
    let tx_object = TransactionParameters {
        nonce: Some(U256::from(nonce)),
        to: Some(to),
        gas: gas_limit,
        gas_price: None,
        value: U256::from(0),
        data: Default::default(),
        chain_id: Some(chain_id),
        transaction_type: Some(U64::from(2)),
        access_list: None,
        max_fee_per_gas: Some(max_fee_per_gas),
        priority_fee: Some(priority_fee),
    };
    tx_object
}

/// Below sends a transaction to a local node that stores private keys (eg Ganache)
/// For generating and signing a transaction offline, before transmitting it to a public node (eg Infura) see transaction_public
#[tokio::main]
async fn main() -> web3::Result {
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

    let nonce = get_transaction_count(from_addr, &web3, false).await.unwrap();
    println!("nonce: {}", nonce);


    let priority_fee = if chain_id == 5 {
        1_100_000_000_u64
    } else {
        panic!("Chain ID not supported");
    };

    let erc20_tx_dao = Erc20TransactionDao {
        from: format!("{from_addr:#x}"),
        to: format!("{to:#x}"),
        token: "0x0000000000000000000000000000000000000000".to_string(),
        chain_id: chain_id,
        gas_limit: 80000,
        max_fee_per_gas: U256::from(1000_000_000_000_u64).to_string(),
        priority_fee: priority_fee.to_string(),
        amount: "1000000000000000000".to_string(),
        nonce: nonce,
    };

    println!("erc20_tx_dao: {:?}", erc20_tx_dao);


    println!("tx_object: {:?}", tx_object);

    // Sign the tx (can be done offline)
    let signed = web3
        .accounts()
        .sign_transaction(tx_object, &secret_key)
        .await?;


    println!("Transaction hash: {:?}", signed.transaction_hash);
    println!("Transaction payload: {:?}", signed.raw_transaction);


    // Send the tx to infura
    let result = web3
        .eth()
        .send_raw_transaction(signed.raw_transaction)
        .await?;

    println!("Tx succeeded with hash: {:#x}", result);

    Ok(())
}
