use lazy_static::lazy_static;
use std::error;
use std::str::FromStr;
use web3::contract::tokens::Tokenize;
use web3::contract::Contract;
use web3::transports::Http;
use web3::types::{Address, U256};
use web3::{Transport, Web3};

lazy_static! {
    pub static ref DUMMY_RPC_PROVIDER: Web3<Http> = {
        let transport = web3::transports::Http::new("http://noconn").unwrap();
        Web3::new(transport)
    };
    pub static ref ERC20_CONTRACT_TEMPLATE: Contract<Http> =
        prepare_contract_template(include_bytes!("../contracts/ierc20.json")).unwrap();
    pub static ref ERC20_MULTI_CONTRACT_TEMPLATE: Contract<Http> = {
        prepare_contract_template(include_bytes!("../contracts/multi_transfer_erc20.json")).unwrap()
    };
    pub static ref MULTI_ERC20_MUMBAI: Address =
        Address::from_str("0x800010D7d0d315DCA795110ecCf0127cBd76b89f").unwrap();
    pub static ref MULTI_ERC20_GOERLI: Address =
        Address::from_str("0x7777784f803a7bf1d7f115f849d29ce5706da64a").unwrap();
    pub static ref MULTI_ERC20_POLYGON: Address =
        Address::from_str("0x50100d4faf5f3b09987dea36dc2eddd57a3e561b").unwrap();
}

pub fn prepare_contract_template(json_abi: &[u8]) -> Result<Contract<Http>, Box<dyn error::Error>> {
    let contract = Contract::from_json(
        DUMMY_RPC_PROVIDER.eth(),
        Address::from_str("0x0000000000000000000000000000000000000000").unwrap(),
        json_abi,
    )?;

    Ok(contract)
}

pub fn contract_encode<P, T>(
    contract: &Contract<T>,
    func: &str,
    params: P,
) -> Result<Vec<u8>, web3::ethabi::Error>
where
    P: Tokenize,
    T: Transport,
{
    contract
        .abi()
        .function(func)
        .and_then(|function| function.encode_input(&params.into_tokens()))
}

#[allow(dead_code)]
pub fn get_erc20_balance_of(address: Address) -> Result<Vec<u8>, web3::ethabi::Error> {
    contract_encode(
        &ERC20_CONTRACT_TEMPLATE,
        "balance_of",
        (address.to_string(),),
    )
}

pub fn get_erc20_transfer(address: Address, amount: U256) -> Result<Vec<u8>, web3::ethabi::Error> {
    contract_encode(&ERC20_CONTRACT_TEMPLATE, "transfer", (address, amount))
}
