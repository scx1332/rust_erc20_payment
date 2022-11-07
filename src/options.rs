use std::ops::Add;
use std::str::FromStr;
use structopt::StructOpt;
use web3::types::{Address, U256};


#[derive(Debug, StructOpt)]
#[structopt(name = "example", about = "An example of StructOpt usage.")]
pub struct CliOptions {
    #[structopt(long = "receiver", help = "Receiver address, or coma separated list of receivers")]
    receivers: String,

    #[structopt(long = "amount", help = "Amount, or coma separated list of amounts")]
    amounts: String,

    #[structopt(long = "chain-id", default_value = "80001")]
    chain_id: i64,

    #[structopt(long = "token-addr", help = "Token address, if not set, ETH will be used") ]
    token_addr: Option<String>,

    #[structopt(long = "memory-db", help = "Use memory db, default is file db") ]
    memory_db: bool,
}

pub struct ValidatedOptions {
    pub receivers: Vec<Address>,
    pub amounts: Vec<U256>,
    pub chain_id: i64,
    pub token_addr: Option<Address>,
    pub memory_db: bool,
}


pub fn validated_cli() -> Result<ValidatedOptions, Box<dyn std::error::Error>> {
    let opt: CliOptions = CliOptions::from_args();

    let amounts = opt.amounts.split(",").map(|s| U256::from_dec_str(s).unwrap()).collect();
    let receivers = opt.receivers.split(",").map(|s| Address::from_str(s).unwrap()).collect();
    let token_addr = opt.token_addr.map(|s| Address::from_str(&s)).transpose()?;

    Ok(ValidatedOptions {
        receivers,
        amounts,
        chain_id: opt.chain_id,
        token_addr,
        memory_db: opt.memory_db,
    })
}