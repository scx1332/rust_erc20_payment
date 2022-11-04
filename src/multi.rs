use sqlx::encode::IsNull::No;
use web3::transports::Http;
use web3::types::{Address, Bytes, CallRequest, U256};
use web3::Web3;
use crate::model::Web3TransactionDao;
use crate::contracts::get_erc20_allowance;
use crate::{error::PaymentError};


pub async fn contract_approve(web3: &Web3<Http>, owner: Address, token: Address, spender: Address) -> Result<(), PaymentError> {
    log::debug!("Checking multi payment contract for allowance...");
    let call_request = CallRequest {
        from: Some(owner),
        to: Some(token),
        gas: None,
        gas_price: None,
        value: None,
        data: Some(Bytes(get_erc20_allowance(owner, spender)?)),
        transaction_type: None,
        access_list: None,
        max_fee_per_gas: None,
        max_priority_fee_per_gas: None,
    };
    let res = web3.eth().call(call_request, None).await?;
    if res.0.len() != 32 {
        return Err(PaymentError::OtherError("Invalid response from ERC20 allowance check".to_string()));
    };
    let allowance = U256::from_big_endian(&res.0);
    log::info!("Allowance: owner: {:?}, token: {:?}, contract: {:?}, allowance: {:?}", owner, token, spender, allowance);

    Ok(())
}

pub async fn check_allowance(web3: &Web3<Http>, owner: Address, token: Address, spender: Address) -> Result<U256, PaymentError> {
    log::debug!("Checking multi payment contract for allowance...");
    let call_request = CallRequest {
        from: Some(owner),
        to: Some(token),
        gas: None,
        gas_price: None,
        value: None,
        data: Some(Bytes(get_erc20_allowance(owner, spender)?)),
        transaction_type: None,
        access_list: None,
        max_fee_per_gas: None,
        max_priority_fee_per_gas: None,
    };
    let res = web3.eth().call(call_request, None).await?;
    if res.0.len() != 32 {
        return Err(PaymentError::OtherError("Invalid response from ERC20 allowance check".to_string()));
    };
    let allowance = U256::from_big_endian(&res.0);
    log::info!("Allowance: owner: {:?}, token: {:?}, contract: {:?}, allowance: {:?}", owner, token, spender, allowance);

    Ok(allowance)
}