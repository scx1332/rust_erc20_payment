use thiserror::Error;
use web3::ethabi::ethereum_types::FromDecStrErr;
use web3::types::U256;

#[derive(Debug)]
pub struct AllowanceRequest {
    pub owner: String,
    pub token_addr: String,
    pub spender_addr: String,
    pub chain_id: i64,
    pub amount: U256,
}

#[derive(Error, Debug)]
pub enum PaymentError {
    #[error("Hex conversion error: {0}")]
    HexError(#[from] rustc_hex::FromHexError),
    #[error("Dec conversion error: {0}")]
    DecError(#[from] FromDecStrErr),
    #[error("sqlx error: {0}")]
    SqlxError(#[from] sqlx::Error),
    #[error("conversion error: {0}")]
    ConversionError(#[from] crate::utils::ConversionError),
    #[error("web3 error: {0}")]
    Web3Error(#[from] web3::Error),
    #[error("hex conversion error: {0}")]
    Web3AbiError(#[from] web3::ethabi::Error),
    #[error("Io error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("Other error: {0}")]
    OtherError(String),
    #[error("Transaction failed error: {0}")]
    TransactionFailedError(String),
    #[error("No allowance found for chain id: {0:?}")]
    NoAllowanceFound(AllowanceRequest),
}
