use crate::error::PaymentError;
use std::str::FromStr;
use structopt::StructOpt;
use web3::types::{Address, U256};

#[allow(unused)]
#[derive(Debug, StructOpt)]
#[structopt(name = "example", about = "An example of StructOpt usage.")]
pub struct CliOptions {
    #[structopt(
        long = "receivers",
        help = "Receiver address, or coma separated list of receivers"
    )]
    receivers: String,

    #[structopt(long = "amounts", help = "Amount, or coma separated list of amounts")]
    amounts: String,

    #[structopt(long = "chain-id", default_value = "80001")]
    chain_id: i64,

    #[structopt(
        long = "token-addr",
        help = "Token address, if not set, ETH will be used"
    )]
    token_addr: Option<String>,

    #[structopt(long = "plain-eth", help = "Set if you want to send main token")]
    plain_eth: bool,

    #[structopt(long = "memory-db", help = "Use memory db, default is file db")]
    memory_db: bool,
}

#[allow(unused)]
pub struct ValidatedOptions {
    pub receivers: Vec<Address>,
    pub amounts: Vec<U256>,
    pub chain_id: i64,
    pub token_addr: Option<Address>,
    pub memory_db: bool,
}

#[allow(unused)]
pub fn validated_cli() -> Result<ValidatedOptions, PaymentError> {
    let opt: CliOptions = CliOptions::from_args();

    let split_pattern = [',', ';'];
    let mut amounts = Vec::<U256>::new();
    for amount in opt.amounts.split(&split_pattern) {
        let amount = U256::from_dec_str(amount).map_err(|_| {
            PaymentError::OtherError(format!("Invalid amount when parsing input: {amount}"))
        })?;
        amounts.push(amount);
    }

    let mut receivers = Vec::<Address>::new();
    for receiver in opt.receivers.split(&split_pattern) {
        let receiver = Address::from_str(receiver).map_err(|_| {
            PaymentError::OtherError(format!("Invalid receiver when parsing input: {receiver}"))
        })?;
        receivers.push(receiver);
    }

    if receivers.len() != amounts.len() {
        return Err(PaymentError::OtherError(format!(
            "Receivers count and amount count don't match: {} != {}",
            receivers.len(),
            amounts.len()
        )));
    };
    if receivers.is_empty() {
        return Err(PaymentError::OtherError(format!("No receivers specified")));
    };
    if opt.plain_eth && opt.token_addr.is_some() {
        return Err(PaymentError::OtherError(format!(
            "Can't specify both plain-eth and token-addr"
        )));
    };
    if !opt.plain_eth && opt.token_addr.is_none() {
        return Err(PaymentError::OtherError(format!(
            "Specify token-addr or set plain-eth true to plain transfer"
        )));
    };


    let token_addr = if opt.plain_eth {
        None
    } else {
        opt.token_addr.map(|s| Address::from_str(&s)).transpose()?
    };

    Ok(ValidatedOptions {
        receivers,
        amounts,
        chain_id: opt.chain_id,
        token_addr,
        memory_db: opt.memory_db,
    })
}
