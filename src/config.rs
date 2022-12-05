use serde::Deserialize;
use std::collections::btree_map::BTreeMap as Map;

use std::fs;
use std::path::Path;

use crate::error::PaymentError;

use crate::error::CustomError;
use crate::error::ErrorBag;
use crate::{err_custom_create, err_from};
use web3::types::Address;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub chain: Map<String, Chain>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct MultiContractSettings {
    pub address: Address,
    pub max_at_once: usize,
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
    pub transaction_timeout: u64,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Token {
    pub symbol: Option<String>,
    pub address: Address,
    pub faucet: Option<Address>,
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, PaymentError> {
        match toml::from_slice(&fs::read(path).map_err(err_from!())?) {
            Ok(config) => Ok(config),
            Err(e) => Err(err_custom_create!("Failed to parse toml {:?}", e)),
        }
    }
}
