use thiserror::Error;
use web3::ethabi::ethereum_types::FromDecStrErr;

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
    #[error("Parsing error: {0}")]
    ParsingError(String),
    #[error("Other error: {0}")]
    OtherError(String),
}
