use crate::config::Config;
use crate::error::PaymentError;
use crate::utils::gwei_to_u256;
use rand::Rng;
use std::collections::BTreeMap;
use web3::transports::Http;
use web3::types::{Address, U256};
use web3::Web3;

#[derive(Clone, Debug)]
pub struct ProviderSetup {
    pub provider: Web3<Http>,
    pub number_of_calls: u64,
}

#[derive(Clone, Debug)]
pub struct ChainSetup {
    pub providers: Vec<ProviderSetup>,
    pub max_fee_per_gas: U256,
    pub priority_fee: U256,
    pub glm_address: Option<Address>,
    pub multi_contract_address: Option<Address>,
    pub transaction_timeout: u64,
}

#[derive(Clone, Debug)]
pub struct PaymentSetup {
    pub chain_setup: BTreeMap<usize, ChainSetup>,
}

impl PaymentSetup {
    pub fn new(config: &Config) -> Result<Self, PaymentError> {
        let mut ps = PaymentSetup {
            chain_setup: BTreeMap::new(),
        };
        for chain_config in &config.chain {
            let mut providers = Vec::new();
            for endp in &chain_config.1.rpc_endpoints {
                let Ok(transport) = web3::transports::Http::new(&endp) else {
                    return Err(PaymentError::OtherError(format!("Failed to create transport for endpoint: {}", endp)));
                };
                let provider = Web3::new(transport);
                providers.push(ProviderSetup {
                    provider,
                    number_of_calls: 0,
                });
            }
            ps.chain_setup.insert(
                chain_config.1.network_id,
                ChainSetup {
                    providers,
                    max_fee_per_gas: gwei_to_u256(chain_config.1.max_fee_per_gas)?,
                    priority_fee: gwei_to_u256(chain_config.1.priority_fee)?,
                    glm_address: chain_config.1.token.clone().map(|t| t.address),
                    multi_contract_address: chain_config
                        .1
                        .multi_contract
                        .clone()
                        .map(|m| m.address),
                    transaction_timeout: chain_config.1.transaction_timeout,
                },
            );
        }
        Ok(ps)
    }
    pub fn get_chain_setup(&self, chain_id: i64) -> Result<&ChainSetup, PaymentError> {
        self.chain_setup
            .get(&(chain_id as usize))
            .ok_or(PaymentError::OtherError(format!(
                "No chain setup for chain id: {}",
                chain_id
            )))
    }

    pub fn get_provider(&self, chain_id: i64) -> Result<&Web3<Http>, PaymentError> {
        let chain_setup =
            self.chain_setup
                .get(&(chain_id as usize))
                .ok_or(PaymentError::OtherError(format!(
                    "No chain setup for chain id: {}",
                    chain_id
                )))?;

        let mut rng = rand::thread_rng();
        let provider = chain_setup
            .providers
            .get(rng.gen_range(0..chain_setup.providers.len()))
            .ok_or(PaymentError::OtherError(format!(
                "No providers found for chain id: {}",
                chain_id
            )))?;
        Ok(&provider.provider)
    }
}
