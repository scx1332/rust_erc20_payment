use crate::contracts::{get_erc20_transfer, get_multi_direct_packed};
use crate::model::TokenTransfer;
use crate::model::Web3TransactionDao;

use secp256k1::SecretKey;

use crate::contracts::get_erc20_approve;
use crate::error::PaymentError;
use crate::utils::ConversionError;
use std::str::FromStr;
use web3::transports::Http;
use web3::types::{Address, Bytes, CallRequest, TransactionId, TransactionParameters, U256, U64};
use web3::Web3;
use crate::multi::pack_transfers_for_multi_contract;

fn decode_data_to_bytes(web3_tx_dao: &Web3TransactionDao) -> Result<Option<Bytes>, PaymentError> {
    Ok(if let Some(data) = &web3_tx_dao.call_data {
        let hex_data = hex::decode(data)
            .map_err(|_err| ConversionError::from("Failed to convert data from hex".into()))?;
        Some(Bytes(hex_data))
    } else {
        None
    })
}

pub fn dao_to_call_request(web3_tx_dao: &Web3TransactionDao) -> Result<CallRequest, PaymentError> {
    Ok(CallRequest {
        from: Some(Address::from_str(&web3_tx_dao.from_addr)?),
        to: Some(Address::from_str(&web3_tx_dao.to_addr)?),
        gas: Some(U256::from(web3_tx_dao.gas_limit)),
        gas_price: None,
        value: Some(U256::from_dec_str(&web3_tx_dao.val)?),
        data: decode_data_to_bytes(web3_tx_dao)?,
        transaction_type: Some(U64::from(2)),
        access_list: None,
        max_fee_per_gas: Some(U256::from_dec_str(&web3_tx_dao.max_fee_per_gas)?),
        max_priority_fee_per_gas: Some(U256::from_dec_str(&web3_tx_dao.priority_fee)?),
    })
}

pub fn dao_to_transaction(
    web3_tx_dao: &Web3TransactionDao,
) -> Result<TransactionParameters, PaymentError> {
    Ok(TransactionParameters {
        nonce: Some(U256::from(
            web3_tx_dao
                .nonce
                .ok_or(PaymentError::OtherError("Missing nonce".into()))?,
        )),
        to: Some(Address::from_str(&web3_tx_dao.to_addr)?),
        gas: U256::from(web3_tx_dao.gas_limit),
        gas_price: None,
        value: U256::from_dec_str(&web3_tx_dao.val)?,
        data: decode_data_to_bytes(web3_tx_dao)?.unwrap_or(Bytes::default()),
        chain_id: Some(web3_tx_dao.chain_id as u64),
        transaction_type: Some(U64::from(2)),
        access_list: None,
        max_fee_per_gas: Some(U256::from_dec_str(&web3_tx_dao.max_fee_per_gas)?),
        max_priority_fee_per_gas: Some(U256::from_dec_str(&web3_tx_dao.priority_fee)?),
    })
}

// token_addr NULL means standard (non ERC20) transfer of main chain currency (i.e ETH)
pub fn create_token_transfer(
    from: Address,
    receiver: Address,
    chain_id: u64,
    token_addr: Option<Address>,
    token_amount: U256,
) -> TokenTransfer {
    TokenTransfer {
        id: 0,
        from_addr: format!("{:#x}", from),
        receiver_addr: format!("{:#x}", receiver),
        chain_id: chain_id as i64,
        token_addr: token_addr.map(|addr| format!("{:#x}", addr)),
        token_amount: token_amount.to_string(),
        tx_id: None,
        fee_paid: None,
        error: None,
    }
}

#[allow(dead_code)]
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
        id: 0,
        method: "transfer".to_string(),
        from_addr: format!("{:#x}", from),
        to_addr: format!("{:#x}", to),
        chain_id: chain_id as i64,
        gas_limit: gas_limit as i64,
        max_fee_per_gas: max_fee_per_gas.to_string(),
        priority_fee: priority_fee.to_string(),
        val: amount.to_string(),
        nonce: None,
        processing: 1,
        call_data: None,
        signed_raw_data: None,
        created_date: chrono::Utc::now(),
        first_processed: None,
        signed_date: None,
        broadcast_date: None,
        broadcast_count: 0,
        tx_hash: None,
        confirm_date: None,
        block_number: None,
        chain_status: None,
        fee_paid: None,
        error: None,
    };
    web3_tx_dao
}

#[allow(dead_code)]
pub fn create_eth_transfer_str(
    from_addr: String,
    to_addr: String,
    chain_id: u64,
    gas_limit: u64,
    max_fee_per_gas: String,
    priority_fee: String,
    amount: String,
) -> Web3TransactionDao {
    let web3_tx_dao = Web3TransactionDao {
        id: 0,
        method: "transfer".to_string(),
        from_addr,
        to_addr,
        chain_id: chain_id as i64,
        gas_limit: gas_limit as i64,
        max_fee_per_gas,
        priority_fee,
        val: amount,
        nonce: None,
        processing: 1,
        call_data: None,
        signed_raw_data: None,
        created_date: chrono::Utc::now(),
        first_processed: None,
        signed_date: None,
        broadcast_date: None,
        broadcast_count: 0,
        tx_hash: None,
        confirm_date: None,
        block_number: None,
        chain_status: None,
        fee_paid: None,
        error: None,
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
) -> Result<Web3TransactionDao, PaymentError> {
    Ok(Web3TransactionDao {
        id: 0,
        method: "ERC20.transfer".to_string(),
        from_addr: format!("{:#x}", from),
        to_addr: format!("{:#x}", token),
        chain_id: chain_id as i64,
        gas_limit: gas_limit as i64,
        max_fee_per_gas: max_fee_per_gas.to_string(),
        priority_fee: priority_fee.to_string(),
        val: "0".to_string(),
        nonce: None,
        processing: 1,
        call_data: Some(hex::encode(get_erc20_transfer(erc20_to, erc20_amount)?)),
        signed_raw_data: None,
        created_date: chrono::Utc::now(),
        first_processed: None,
        signed_date: None,
        broadcast_date: None,
        broadcast_count: 0,
        tx_hash: None,
        confirm_date: None,
        block_number: None,
        chain_status: None,
        fee_paid: None,
        error: None,
    })
}

pub fn create_erc20_transfer_multi(
    from: Address,
    contract: Address,
    erc20_to: Address,
    erc20_amount: U256,
    chain_id: u64,
    gas_limit: u64,
    max_fee_per_gas: U256,
    priority_fee: U256,
) -> Result<Web3TransactionDao, PaymentError> {
    let packed = pack_transfers_for_multi_contract(
        vec![erc20_to],
        vec![erc20_amount],
    )?;
    Ok(Web3TransactionDao {
        id: 0,
        method: "ERC20.golemTransferDirectPacked".to_string(),
        from_addr: format!("{:#x}", from),
        to_addr: format!("{:#x}", contract),
        chain_id: chain_id as i64,
        gas_limit: gas_limit as i64,
        max_fee_per_gas: max_fee_per_gas.to_string(),
        priority_fee: priority_fee.to_string(),
        val: "0".to_string(),
        nonce: None,
        processing: 1,
        call_data: Some(hex::encode(get_multi_direct_packed(packed)?)),
        signed_raw_data: None,
        created_date: chrono::Utc::now(),
        first_processed: None,
        signed_date: None,
        broadcast_date: None,
        broadcast_count: 0,
        tx_hash: None,
        confirm_date: None,
        block_number: None,
        chain_status: None,
        fee_paid: None,
        error: None,
    })
}


pub fn create_erc20_approve(
    from: Address,
    token: Address,
    contract_to_approve: Address,
    chain_id: u64,
    gas_limit: u64,
    max_fee_per_gas: U256,
    priority_fee: U256,
) -> Result<Web3TransactionDao, PaymentError> {
    Ok(Web3TransactionDao {
        id: 0,
        method: "ERC20.approve".to_string(),
        from_addr: format!("{:#x}", from),
        to_addr: format!("{:#x}", token),
        chain_id: chain_id as i64,
        gas_limit: gas_limit as i64,
        max_fee_per_gas: max_fee_per_gas.to_string(),
        priority_fee: priority_fee.to_string(),
        val: "0".to_string(),
        nonce: None,
        processing: 1,
        call_data: Some(hex::encode(get_erc20_approve(
            contract_to_approve,
            U256::max_value(),
        )?)),
        signed_raw_data: None,
        created_date: chrono::Utc::now(),
        first_processed: None,
        signed_date: None,
        broadcast_date: None,
        broadcast_count: 0,
        tx_hash: None,
        confirm_date: None,
        block_number: None,
        chain_status: None,
        fee_paid: None,
        error: None,
    })
}

pub async fn check_transaction(
    web3: &Web3<Http>,
    web3_tx_dao: &mut Web3TransactionDao,
) -> Result<(), PaymentError> {
    log::error!(
        "check_transaction: {:?}",
        dao_to_call_request(&web3_tx_dao).unwrap()
    );
    let gas_est = web3
        .eth()
        .estimate_gas(dao_to_call_request(&web3_tx_dao)?, None)
        .await?;

    let add_gas_safety_margin: U256 = U256::from(20000);
    let gas_limit = gas_est + U256::from(add_gas_safety_margin);
    println!("Set gas limit basing on gas estimation: {gas_est}. Setting {gas_limit} increased by {add_gas_safety_margin} for safe execution.");
    web3_tx_dao.gas_limit = gas_limit.as_u64() as i64;

    Ok(())
}

pub async fn sign_transaction(
    web3: &Web3<Http>,
    web3_tx_dao: &mut Web3TransactionDao,
    secret_key: &SecretKey,
) -> Result<(), PaymentError> {
    let tx_object = dao_to_transaction(&web3_tx_dao)?;
    log::debug!("Signing transaction: {:#?}", tx_object);
    // Sign the tx (can be done offline)
    let signed = web3
        .accounts()
        .sign_transaction(tx_object, secret_key)
        .await?;

    let slice: Vec<u8> = signed.raw_transaction.0;
    web3_tx_dao.signed_raw_data = Some(format!("{}", hex::encode(slice)));
    web3_tx_dao.signed_date = Some(chrono::Utc::now());
    web3_tx_dao.tx_hash = Some(format!("{:#x}", signed.transaction_hash));
    log::debug!("Transaction signed successfully: {:#?}", web3_tx_dao);
    Ok(())
}

pub async fn send_transaction(
    web3: &Web3<Http>,
    web3_tx_dao: &mut Web3TransactionDao,
) -> Result<(), PaymentError> {
    if let Some(signed_raw_data) = web3_tx_dao.signed_raw_data.as_ref() {
        let bytes =
            Bytes(hex::decode(&signed_raw_data).map_err(|_err| {
                ConversionError::from("cannot decode signed_raw_data".to_string())
            })?);
        let result = web3.eth().send_raw_transaction(bytes).await;
        web3_tx_dao.broadcast_date = Some(chrono::Utc::now());
        if let Err(e) = result {
            log::error!("Error sending transaction: {:#?}", e);
        }
    } else {
        return Err(PaymentError::OtherError("No signed raw data".into()));
    }
    Ok(())
}

// it seems that this function is not needed at all for checking the transaction status
// instead use nonce and transaction receipt
#[allow(unused)]
pub async fn find_tx(
    web3: &Web3<Http>,
    web3_tx_dao: &mut Web3TransactionDao,
) -> Result<bool, PaymentError> {
    if let Some(tx_hash) = web3_tx_dao.tx_hash.as_ref() {
        let tx_hash = web3::types::H256::from_str(tx_hash)
            .map_err(|err| ConversionError::from("Failed to convert tx hash".into()))?;
        let tx = web3.eth().transaction(TransactionId::Hash(tx_hash)).await?;
        if let Some(tx) = tx {
            web3_tx_dao.block_number = tx.block_number.map(|x| x.as_u64() as i64);
            return Ok(true);
        } else {
            return Ok(false);
        }
    } else {
        return Err(PaymentError::OtherError("No tx hash".into()));
    }
}

pub async fn find_receipt(
    web3: &Web3<Http>,
    web3_tx_dao: &mut Web3TransactionDao,
) -> Result<bool, PaymentError> {
    if let Some(tx_hash) = web3_tx_dao.tx_hash.as_ref() {
        let tx_hash = web3::types::H256::from_str(tx_hash)
            .map_err(|_err| ConversionError::from("Cannot parse tx_hash".to_string()))?;
        let receipt = web3.eth().transaction_receipt(tx_hash).await?;
        if let Some(receipt) = receipt {
            web3_tx_dao.block_number = receipt.block_number.map(|x| x.as_u64() as i64);
            web3_tx_dao.chain_status = receipt.status.map(|x| x.as_u64() as i64);
            log::warn!("receipt: {:?}", receipt);

            let gas_used = receipt
                .gas_used
                .ok_or(PaymentError::OtherError("Gas used expected".into()))?;
            let effective_gas_price = receipt.effective_gas_price.ok_or(
                PaymentError::OtherError("Effective gas price expected".into()),
            )?;
            web3_tx_dao.fee_paid = Some((gas_used * effective_gas_price).to_string());
            return Ok(true);
        } else {
            web3_tx_dao.block_number = None;
            web3_tx_dao.chain_status = None;
            web3_tx_dao.fee_paid = None;
            return Ok(false);
        }
    } else {
        return Err(PaymentError::OtherError("No tx hash".into()));
    }
}
