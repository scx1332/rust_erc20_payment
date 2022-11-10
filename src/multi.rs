use web3::transports::Http;
use web3::types::{Address, Bytes, CallRequest, U256};
use web3::Web3;

use crate::contracts::get_erc20_allowance;
use crate::error::PaymentError;

pub async fn check_allowance(
    web3: &Web3<Http>,
    owner: Address,
    token: Address,
    spender: Address,
) -> Result<U256, PaymentError> {
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
        return Err(PaymentError::OtherError(
            format!("Invalid response from ERC20 allowance check {:?}", res),
        ));
    };
    let allowance = U256::from_big_endian(&res.0);
    log::info!(
        "Allowance: owner: {:?}, token: {:?}, contract: {:?}, allowance: {:?}",
        owner,
        token,
        spender,
        allowance
    );

    Ok(allowance)
}


pub fn pack_transfers_for_multi_contract(receivers: Vec<Address>, amounts: Vec<U256>) -> Result<Vec<[u8; 32]>, PaymentError> {
    let max_value = U256::MAX / ( U256::from(2).pow(U256::from(160)) );
    log::debug!("Max value for pack transfers: {:?}", max_value.to_string());
    for amount in &amounts {
        if amount > &max_value {
            return Err(PaymentError::OtherError(
                "Amount is too big to use packed error".to_string(),
            ));
        }
    }

    let packed: Vec<[u8; 32]> = receivers
        .iter()
        .zip(amounts.iter())
        .map(|(&receiver, &amount)| {
            let mut packet2 = [0u8; 32];
            amount.to_big_endian(&mut packet2[..]);
            packet2[..20].copy_from_slice(&receiver[..20]);
            packet2
        })
        .collect();
    //log::debug!("Packed: {:?}", packed);
    Ok(packed)
}