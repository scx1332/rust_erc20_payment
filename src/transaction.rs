use crate::contracts::get_erc20_transfer;
use crate::model::Web3TransactionDao;
use secp256k1::SecretKey;
use std::error;
use std::str::FromStr;
use rand::distributions::{Alphanumeric, DistString};
use web3::transports::Http;
use web3::types::{Address, Bytes, CallRequest, TransactionId, TransactionParameters, U256, U64};
use web3::Web3;

fn decode_data_to_bytes(web3_tx_dao: &Web3TransactionDao) -> Result<Bytes, Box<dyn error::Error>> {
    Ok(if let Some(data) = &web3_tx_dao.data {
        let hex_data = hex::decode(data)?;
        Bytes(hex_data)
    } else {
        Bytes::default()
    })
}

pub fn dao_to_call_request(
    web3_tx_dao: &Web3TransactionDao,
) -> Result<CallRequest, Box<dyn error::Error>> {
    Ok(CallRequest {
        from: Some(Address::from_str(&web3_tx_dao.from)?),
        to: Some(Address::from_str(&web3_tx_dao.to)?),
        gas: Some(U256::from(web3_tx_dao.gas_limit)),
        gas_price: None,
        value: Some(U256::from_dec_str(&web3_tx_dao.value)?),
        data: Some(decode_data_to_bytes(web3_tx_dao)?),
        transaction_type: Some(U64::from(2)),
        access_list: None,
        max_fee_per_gas: Some(U256::from_dec_str(&web3_tx_dao.max_fee_per_gas)?),
        max_priority_fee_per_gas: Some(U256::from_dec_str(&web3_tx_dao.priority_fee)?),
    })
}

pub fn dao_to_transaction(
    web3_tx_dao: &Web3TransactionDao,
) -> Result<TransactionParameters, Box<dyn error::Error>> {
    Ok(TransactionParameters {
        nonce: Some(U256::from(web3_tx_dao.nonce.ok_or("Missing nonce")?)),
        to: Some(Address::from_str(&web3_tx_dao.to)?),
        gas: U256::from(web3_tx_dao.gas_limit),
        gas_price: None,
        value: U256::from_dec_str(&web3_tx_dao.value)?,
        data: decode_data_to_bytes(web3_tx_dao)?,
        chain_id: Some(web3_tx_dao.chain_id),
        transaction_type: Some(U64::from(2)),
        access_list: None,
        max_fee_per_gas: Some(U256::from_dec_str(&web3_tx_dao.max_fee_per_gas)?),
        max_priority_fee_per_gas: Some(U256::from_dec_str(&web3_tx_dao.priority_fee)?),
    })
}

pub fn get_unique_id() -> String {
    let string = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);
    string
}

pub fn create_eth_transfer(
    from: Address,
    to: Address,
    chain_id: u64,
    gas_limit: u64,
    max_fee_per_gas: U256,
    priority_fee: U256,
    amount: U256,
) -> Web3TransactionDao {
    let web3_tx_dao = Web3TransactionDao {
        id: get_unique_id(),
        from: format!("{:#x}", from),
        to: format!("{:#x}", to),
        chain_id,
        gas_limit,
        max_fee_per_gas: max_fee_per_gas.to_string(),
        priority_fee: priority_fee.to_string(),
        value: amount.to_string(),
        nonce: None,
        data: None,
        signed_raw_data: None,
        created_date: chrono::Utc::now(),
        signed_date: None,
        broadcast_date: None,
        tx_hash: None,
        confirmed_date: None,
        block_number: None,
        chain_status: None,
        fee_paid: None,
    };
    web3_tx_dao
}

pub fn create_eth_transfer_str(
    from: String,
    to: String,
    chain_id: u64,
    gas_limit: u64,
    max_fee_per_gas: String,
    priority_fee: String,
    amount: String,
) -> Web3TransactionDao {
    let web3_tx_dao = Web3TransactionDao {
        id: get_unique_id(),
        from,
        to,
        chain_id,
        gas_limit,
        max_fee_per_gas,
        priority_fee,
        value: amount,
        nonce: None,
        data: None,
        signed_raw_data: None,
        created_date: chrono::Utc::now(),
        signed_date: None,
        broadcast_date: None,
        tx_hash: None,
        confirmed_date: None,
        block_number: None,
        chain_status: None,
        fee_paid: None,
    };
    web3_tx_dao
}

pub fn create_erc20_transfer(
    from: Address,
    token: Address,
    erc20_to: Address,
    erc20_amount: U256,
    chain_id: u64,
    gas_limit: u64,
    max_fee_per_gas: U256,
    priority_fee: U256,
) -> Result<Web3TransactionDao, Box<dyn error::Error>> {
    Ok(Web3TransactionDao {
        id: get_unique_id(),
        from: format!("{:#x}", from),
        to: format!("{:#x}", token),
        chain_id,
        gas_limit,
        max_fee_per_gas: max_fee_per_gas.to_string(),
        priority_fee: priority_fee.to_string(),
        value: "0".to_string(),
        nonce: None,
        data: Some(hex::encode(get_erc20_transfer(erc20_to, erc20_amount)?)),
        signed_raw_data: None,
        created_date: chrono::Utc::now(),
        signed_date: None,
        broadcast_date: None,
        tx_hash: None,
        confirmed_date: None,
        block_number: None,
        chain_status: None,
        fee_paid: None,
    })
}

pub async fn check_transaction(
    web3: &Web3<Http>,
    web3_tx_dao: &mut Web3TransactionDao,
) -> Result<(), Box<dyn error::Error>> {
    let gas_est = web3
        .eth()
        .estimate_gas(dao_to_call_request(&web3_tx_dao)?, None)
        .await?;

    let add_gas_safety_margin: U256 = U256::from(20000);
    let gas_limit = gas_est + U256::from(add_gas_safety_margin);
    println!("Set gas limit basing on gas estimation: {gas_est}. Setting {gas_limit} increased by {add_gas_safety_margin} for safe execution.");
    web3_tx_dao.gas_limit = gas_limit.as_u64();

    Ok(())
}

pub async fn sign_transaction(
    web3: &Web3<Http>,
    web3_tx_dao: &mut Web3TransactionDao,
    secret_key: &SecretKey,
) -> Result<(), Box<dyn error::Error>> {
    let tx_object = dao_to_transaction(&web3_tx_dao)?;
    println!("tx_object: {:?}", tx_object);

    // Sign the tx (can be done offline)
    let signed = web3
        .accounts()
        .sign_transaction(tx_object, secret_key)
        .await?;

    let slice: Vec<u8> = signed.raw_transaction.0;
    web3_tx_dao.signed_raw_data = Some(format!("{}", hex::encode(slice)));
    web3_tx_dao.signed_date = Some(chrono::Utc::now());
    web3_tx_dao.tx_hash = Some(format!("{:#x}", signed.transaction_hash));
    Ok(())
}

pub async fn send_transaction(
    web3: &Web3<Http>,
    web3_tx_dao: &mut Web3TransactionDao,
) -> Result<(), Box<dyn error::Error>> {
    if let Some(signed_raw_data) = web3_tx_dao.signed_raw_data.as_ref() {
        let bytes = Bytes(hex::decode(&signed_raw_data)?);
        let result = web3.eth().send_raw_transaction(bytes).await;
        web3_tx_dao.broadcast_date = Some(chrono::Utc::now());
        if let Err(e) = result {
            println!("Error sending transaction: {:?}", e);
        }
    } else {
        return Err("No signed raw data".into());
    }
    Ok(())
}

// it seems that this function is not needed at all for checking the transaction status
// instead use nonce and transaction receipt
#[allow(unused)]
pub async fn find_tx(
    web3: &Web3<Http>,
    web3_tx_dao: &mut Web3TransactionDao,
) -> Result<bool, Box<dyn error::Error>> {
    if let Some(tx_hash) = web3_tx_dao.tx_hash.as_ref() {
        let tx_hash = web3::types::H256::from_str(tx_hash)?;
        let tx = web3.eth().transaction(TransactionId::Hash(tx_hash)).await?;
        if let Some(tx) = tx {
            web3_tx_dao.block_number = tx.block_number.map(|x| x.as_u64());
            return Ok(true);
        } else {
            return Ok(false);
        }
    } else {
        return Err("No tx hash".into());
    }
}

pub async fn find_receipt(
    web3: &Web3<Http>,
    web3_tx_dao: &mut Web3TransactionDao,
) -> Result<bool, Box<dyn error::Error>> {
    if let Some(tx_hash) = web3_tx_dao.tx_hash.as_ref() {
        let tx_hash = web3::types::H256::from_str(tx_hash)?;
        let receipt = web3.eth().transaction_receipt(tx_hash).await?;
        if let Some(receipt) = receipt {
            web3_tx_dao.block_number = receipt.block_number.map(|x| x.as_u64());
            web3_tx_dao.chain_status = receipt.status.map(|x| x.as_u64());
            web3_tx_dao.fee_paid = Some(
                (receipt.cumulative_gas_used
                    * receipt
                        .effective_gas_price
                        .ok_or("Effective gas price expected")?)
                .to_string(),
            );
            return Ok(true);
        } else {
            web3_tx_dao.block_number = None;
            web3_tx_dao.chain_status = None;
            web3_tx_dao.fee_paid = None;
            return Ok(false);
        }
    } else {
        return Err("No tx hash".into());
    }
}
