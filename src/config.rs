use serde::Deserialize;
use std::collections::btree_map::BTreeMap as Map;

use std::fs;
use std::path::Path;

use crate::error::PaymentError;

use web3::types::Address;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub chain: Map<String, Chain>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct MultiContractSettings {
    pub address: Address,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Chain {
    pub network_id: usize,
    pub rpc_endpoints: Vec<String>,
    pub currency_symbol: Option<String>,
    pub priority_fee: f64,
    pub max_fee_per_gas: f64,
    pub token: Option<Token>,
    pub multi_contract: Option<MultiContractSettings>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Token {
    pub symbol: Option<String>,
    pub address: Address,
    pub faucet: Option<Address>,
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, PaymentError> {
        match toml::from_slice(&fs::read(path)?) {
            Ok(config) => Ok(config),
            Err(e) => Err(PaymentError::OtherError(format!(
                "Failed to parse toml {:?}",
                e
            ))),
        }
    }
}
