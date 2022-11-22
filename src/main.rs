mod config;
mod contracts;
mod db;
mod error;
mod eth;
mod misc;
mod model;
mod multi;
mod options;
mod process;
mod runtime;
mod service;
mod setup;
mod transaction;
mod utils;

use std::fmt;

use web3::contract::Contract;
use web3::transports::Http;

use web3::types::Address;

use crate::error::PaymentError;

use crate::options::validated_cli;
use crate::runtime::start_payment_engine;

struct _Web3ChainConfig {
    glm_token: Address,
    chain_id: u64,
    erc20_contract: Contract<Http>,
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

#[tokio::main]
async fn main() -> Result<(), PaymentError> {
    if let Err(err) = dotenv::dotenv() {
        return Err(PaymentError::OtherError(format!(
            "No .env file found: {}",
            err
        )));
    }
    env_logger::init();
    let cli = validated_cli()?;

    let sp = start_payment_engine(Some(cli)).await?;
    sp.await
        .map_err(|e| PaymentError::OtherError(format!("Service loop failed: {:?}", e)))?;
    Ok(())
}
