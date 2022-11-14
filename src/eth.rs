use secp256k1::{PublicKey, SecretKey};
use sha3::Digest;
use sha3::Keccak256;
use web3::transports::Http;
use web3::types::Address;
use web3::Web3;

pub async fn get_transaction_count(
    address: Address,
    web3: &Web3<Http>,
    pending: bool,
) -> Result<u64, web3::Error> {
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
