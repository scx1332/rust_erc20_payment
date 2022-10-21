use std::error;
use crate::Web3TransactionDao;
use secp256k1::SecretKey;
use std::str::FromStr;
use web3::transports::Http;
use web3::types::{Address, CallRequest, TransactionId, TransactionParameters, U256, U64};
use web3::Web3;

pub fn dao_to_call_request(web3_tx_dao: &Web3TransactionDao) -> CallRequest {
    let _from = Address::from_str(&web3_tx_dao.from[2..]).unwrap();
    let to = Address::from_str(&web3_tx_dao.to[2..]).unwrap();
    // let token = Address::from_str(&web3_tx_dao.token[2..]).unwrap();
    let _chain_id = web3_tx_dao.chain_id;
    let gas_limit = web3_tx_dao.gas_limit;
    let total_fee = U256::from_dec_str(&web3_tx_dao.total_fee).unwrap();
    let priority_fee = U256::from_dec_str(&web3_tx_dao.priority_fee).unwrap();
    let value = U256::from_dec_str(&web3_tx_dao.value).unwrap();
    let _nonce = web3_tx_dao.nonce;

    // Build the tx object
    let call_request = CallRequest {
        from: None,
        to: Some(to),
        gas: Some(U256::from(gas_limit)),
        gas_price: None,
        value: Some(value),
        data: Default::default(),
        transaction_type: Some(U64::from(2)),
        access_list: None,
        max_fee_per_gas: Some(total_fee),
        max_priority_fee_per_gas: Some(priority_fee),
    };
    call_request
}

pub fn dao_to_transaction(web3_tx_dao: &Web3TransactionDao) -> TransactionParameters {
    let _from = Address::from_str(&web3_tx_dao.from[2..]).unwrap();
    let to = Address::from_str(&web3_tx_dao.to[2..]).unwrap();
    // let token = Address::from_str(&web3_tx_dao.token[2..]).unwrap();
    let chain_id = web3_tx_dao.chain_id;
    let gas_limit = web3_tx_dao.gas_limit;
    let total_fee = U256::from_dec_str(&web3_tx_dao.total_fee).unwrap();
    let priority_fee = U256::from_dec_str(&web3_tx_dao.priority_fee).unwrap();
    //let amount = U256::from_dec_str(&web3_tx_dao).unwrap();
    let nonce = web3_tx_dao.nonce;

    // Build the tx object
    let tx_object = TransactionParameters {
        nonce: Some(U256::from(nonce)),
        to: Some(to),
        gas: U256::from(gas_limit),
        gas_price: None,
        value: U256::from(0),
        data: Default::default(),
        chain_id: Some(chain_id),
        transaction_type: Some(U64::from(2)),
        access_list: None,
        max_fee_per_gas: Some(total_fee),
        max_priority_fee_per_gas: Some(priority_fee),
    };
    tx_object
}

pub async fn check_transaction(
    web3: &Web3<Http>,
    web3_tx_dao: &mut Web3TransactionDao,
) -> Result<(), web3::Error> {
    let gas_est = web3
        .eth()
        .estimate_gas(dao_to_call_request(&web3_tx_dao), None)
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
) -> Result<(), web3::Error> {
    let tx_object = dao_to_transaction(&web3_tx_dao);
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
        let bytes = hex::decode(&signed_raw_data)?;
        let result = web3.eth().send_raw_transaction(web3::types::Bytes(bytes)).await;
        web3_tx_dao.broadcast_date = Some(chrono::Utc::now());
        if let Err(e) = result {
            println!("Error sending transaction: {:?}", e);
        }
    } else {
        return Err("No signed raw data".into());
    }
    Ok(())
}

pub async fn find_tx(
    web3: &Web3<Http>,
    web3_tx_dao: &mut Web3TransactionDao,
) -> Result<bool, Box<dyn error::Error>>  {
    if let Some(tx_hash) = web3_tx_dao.tx_hash.as_ref() {
        let tx_hash = web3::types::H256::from_str(tx_hash)?;
        let tx = web3.eth().transaction(TransactionId::Hash(tx_hash)).await?;
        if let Some(tx) = tx {
            web3_tx_dao.block_number = tx.block_number.map(|x| x.as_u64());
            return Ok(true);
        } else {
            return Ok(false);
        }
    }
    else {
        return Err("No tx hash".into());
    }
}
