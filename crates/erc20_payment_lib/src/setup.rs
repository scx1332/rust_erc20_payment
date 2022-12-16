use crate::config::Config;
use crate::error::PaymentError;
use crate::error::{CustomError, ErrorBag};
use crate::eth::get_eth_addr_from_secret;
use crate::utils::gwei_to_u256;
use crate::{err_custom_create, err_from};
use rand::Rng;
use secp256k1::SecretKey;
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
    pub currency_symbol: String,
    pub max_fee_per_gas: U256,
    pub gas_left_warning_limit: u64,
    pub priority_fee: U256,
    pub glm_address: Option<Address>,
    pub multi_contract_address: Option<Address>,
    pub multi_contract_max_at_once: usize,
    pub transaction_timeout: u64,
    pub skip_multi_contract_check: bool,
    pub confirmation_blocks: u64,
}

#[derive(Clone, Debug)]
pub struct PaymentSetup {
    pub chain_setup: BTreeMap<usize, ChainSetup>,
    pub secret_keys: Vec<SecretKey>,
    //pub pub_address: Address,
    pub finish_when_done: bool,
    pub generate_tx_only: bool,
    pub skip_multi_contract_check: bool,
    pub service_sleep: u64,
    pub process_sleep: u64,
}

impl PaymentSetup {
    pub fn new(
        config: &Config,
        secret_keys: Vec<SecretKey>,
        finish_when_done: bool,
        generate_txs_only: bool,
        skip_multi_contract_check: bool,
        service_sleep: u64,
        process_sleep: u64,
    ) -> Result<Self, PaymentError> {
        let mut ps = PaymentSetup {
            chain_setup: BTreeMap::new(),
            secret_keys: secret_keys,
            //pub_address: get_eth_addr_from_secret(secret_key),
            finish_when_done,
            generate_tx_only: generate_txs_only,
            skip_multi_contract_check,
            service_sleep,
            process_sleep,
        };
        for chain_config in &config.chain {
            let mut providers = Vec::new();
            for endp in &chain_config.1.rpc_endpoints {
                let Ok(transport) = web3::transports::Http::new(endp) else {
                    return Err(err_custom_create!("Failed to create transport for endpoint: {}", endp));
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
                    max_fee_per_gas: gwei_to_u256(chain_config.1.max_fee_per_gas)
                        .map_err(err_from!())?,
                    priority_fee: gwei_to_u256(chain_config.1.priority_fee).map_err(err_from!())?,
                    glm_address: chain_config.1.token.clone().map(|t| t.address),
                    multi_contract_address: chain_config
                        .1
                        .multi_contract
                        .clone()
                        .map(|m| m.address),
                    multi_contract_max_at_once: chain_config
                        .1
                        .multi_contract
                        .clone()
                        .map(|m| m.max_at_once)
                        .unwrap_or(1),
                    transaction_timeout: chain_config.1.transaction_timeout,
                    skip_multi_contract_check,
                    confirmation_blocks: chain_config.1.confirmation_blocks,
                    gas_left_warning_limit: chain_config.1.gas_left_warning_limit,
                    currency_symbol: chain_config.1.currency_symbol.clone(),
                },
            );
        }
        Ok(ps)
    }
    pub fn get_chain_setup(&self, chain_id: i64) -> Result<&ChainSetup, PaymentError> {
        self.chain_setup
            .get(&(chain_id as usize))
            .ok_or_else(|| err_custom_create!("No chain setup for chain id: {}", chain_id))
    }

    pub fn get_provider(&self, chain_id: i64) -> Result<&Web3<Http>, PaymentError> {
        let chain_setup = self
            .chain_setup
            .get(&(chain_id as usize))
            .ok_or_else(|| err_custom_create!("No chain setup for chain id: {}", chain_id))?;

        let mut rng = rand::thread_rng();
        let provider = chain_setup
            .providers
            .get(rng.gen_range(0..chain_setup.providers.len()))
            .ok_or_else(|| err_custom_create!("No providers found for chain id: {}", chain_id))?;
        Ok(&provider.provider)
    }
}
