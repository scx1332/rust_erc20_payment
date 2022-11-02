mod contracts;
mod db;
mod eth;
mod model;
mod process;
mod transaction;
mod utils;

use sqlx::{Connection, SqliteConnection};

use secp256k1::{PublicKey, SecretKey};

use std::str::FromStr;

use std::{env, error, fmt};

use crate::transaction::{create_erc20_transfer, create_token_transfer};
use sha3::{Digest, Keccak256};

use web3::contract::Contract;
use web3::transports::Http;

use crate::process::{process_transaction, ProcessTransactionResult};
use crate::utils::gwei_to_u256;
use web3::types::{Address, U256};

use crate::db::create_sqlite_connection;
use crate::db::operations::{
    get_all_processed_transactions, get_all_token_transfers, get_token_transfers_by_tx,
    insert_token_transfer, insert_tx, update_token_transfer, update_tx,
};
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

pub async fn process_transactions(
    conn: &mut SqliteConnection,
    web3: &web3::Web3<web3::transports::Http>,
    secret_key: &SecretKey,
) -> Result<(), Box<dyn error::Error>> {
    loop {
        let mut transactions = get_all_processed_transactions(conn).await?;

        for tx in &mut transactions {
            let process_t_res = process_transaction(tx, web3, secret_key, false).await?;
            match process_t_res {
                ProcessTransactionResult::Confirmed => {
                    tx.processing = 0;

                    let mut db_transaction = conn.begin().await?;
                    let token_transfers =
                        get_token_transfers_by_tx(&mut db_transaction, tx.id).await?;
                    let token_transfers_count = U256::from(token_transfers.len() as u64);
                    for mut token_transfer in token_transfers {
                        if let Some(fee_paid) = tx.fee_paid.clone() {
                            let val = U256::from_dec_str(&fee_paid)?;
                            let val2 = val / token_transfers_count;
                            token_transfer.fee_paid = Some(val2.to_string());
                        } else {
                            token_transfer.fee_paid = None;
                        }
                        update_token_transfer(&mut db_transaction, &token_transfer).await?;
                    }
                    update_tx(&mut db_transaction, tx).await?;
                    db_transaction.commit().await?;
                }
                ProcessTransactionResult::NeedRetry => {
                    tx.processing = 0;

                    let mut db_transaction = conn.begin().await?;
                    let token_transfers =
                        get_token_transfers_by_tx(&mut db_transaction, tx.id).await?;
                    for mut token_transfer in token_transfers {
                        token_transfer.fee_paid = Some("0".to_string());
                        update_token_transfer(&mut db_transaction, &token_transfer).await?;
                    }
                    update_tx(&mut db_transaction, tx).await?;
                    db_transaction.commit().await?;
                }
                ProcessTransactionResult::Unknown => {
                    tx.processing = 1;
                    update_tx(conn, tx).await?;
                }
            }
            //process only one transaction at once
            break;
        }
        if transactions.is_empty() {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
    }
    Ok(())
}

/// Below sends a transaction to a local node that stores private keys (eg Ganache)
/// For generating and signing a transaction offline, before transmitting it to a public node (eg Infura) see transaction_public
#[tokio::main]
async fn main() -> Result<(), Box<dyn error::Error>> {
    env_logger::init();

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
    let to = Address::from_str(&env::var("ETH_TO_ADDRESS").unwrap()).unwrap();

    let (max_fee_per_gas, priority_fee, token_addr) = if chain_id == 5 {
        (
            gwei_to_u256(1000.0)?,
            gwei_to_u256(1.111)?,
            Address::from_str("0x33af15c79d64b85ba14aaffaa4577949104b22e8").unwrap(),
        )
    } else if chain_id == 80001 {
        (
            gwei_to_u256(1000.0)?,
            gwei_to_u256(1.51)?,
            Address::from_str("0x2036807b0b3aaf5b1858ee822d0e111fddac7018").unwrap(),
        )
    } else {
        panic!("Chain ID not supported");
    };

    for transaction_no in 0..2 {
        let token_transfer =
            create_token_transfer(from_addr, to, chain_id, Some(token_addr), U256::from(1 + transaction_no as i32));
        let _token_transfer = insert_token_transfer(&mut conn, &token_transfer).await?;
    };

    for mut token_transfer in get_all_token_transfers(&mut conn).await? {
        if token_transfer.tx_id.is_none() {
            log::debug!("Processing token transfer {:?}", token_transfer);
            let web3_tx_dao = create_erc20_transfer(
                Address::from_str(&token_transfer.from_addr).unwrap(),
                Address::from_str(&token_transfer.token_addr.as_ref().unwrap()).unwrap(),
                Address::from_str(&token_transfer.receiver_addr).unwrap(),
                U256::from_dec_str(&token_transfer.token_amount).unwrap(),
                token_transfer.chain_id as u64,
                1000,
                max_fee_per_gas,
                priority_fee,
            )?;
            let mut tx = conn.begin().await?;
            let web3_tx_dao = insert_tx(&mut tx, &web3_tx_dao).await?;
            token_transfer.tx_id = Some(web3_tx_dao.id);
            update_token_transfer(&mut tx, &token_transfer).await?;
            tx.commit().await?;
        }
    }
    process_transactions(&mut conn, &web3, &secret_key).await?;

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

    //let (res1, res2) = tokio::join!(process_t_res, process_t_res2);
    //println!("Transaction 1: {:?}", res1?);
    // println!("Transaction 2: {:?}", res2?);

    //println!("Transaction hash: {:?}", signed.transaction_hash);
    //println!("Transaction payload: {:?}", signed.raw_transaction);

    //println!("Tx succeeded with hash: {:#x}", result);

    Ok(())
}
