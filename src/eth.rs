use std::error;
use web3::transports::Http;
use web3::types::Address;
use web3::Web3;

pub async fn get_transaction_count(
    address: Address,
    web3: &Web3<Http>,
    pending: bool,
) -> Result<u64, Box<dyn error::Error>> {
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
