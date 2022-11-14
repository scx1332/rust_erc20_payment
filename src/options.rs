use crate::error::PaymentError;
use std::str::FromStr;
use structopt::StructOpt;
use web3::types::{Address, U256};

#[derive(Debug, StructOpt)]
pub struct ProcessOptions {
    #[structopt(
    long = "keep-running",
    help = "Set to keep running when finished processing transactions"
    )]
    keep_running: bool,
}

#[derive(Debug, StructOpt)]
struct TransferOptions {
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

    #[structopt(
        long = "keep-running",
        help = "Set to keep running when finished processing transactions"
    )]
    keep_running: bool,
}

#[derive(Debug, StructOpt)]
struct TestOptions {
    #[structopt(long = "chain-id", default_value = "80001")]
    chain_id: i64,

    #[structopt(long = "generate-count", default_value = "10")]
    generate_count: i64,
}

#[allow(unused)]
#[derive(Debug, StructOpt)]
#[structopt(name = "example", about = "An example of StructOpt usage.")]
enum CliOptions {
    /// Transfer options.
    #[structopt(name = "transfer")]
    Transfer(TransferOptions),
    /// Process options
    #[structopt(name = "process")]
    Process(ProcessOptions),
    /// Test options
    #[structopt(name = "test")]
    Test(TestOptions),
}
#[allow(unused)]
pub struct ValidatedOptions {
    pub receivers: Vec<Address>,
    pub amounts: Vec<U256>,
    pub chain_id: i64,
    pub token_addr: Option<Address>,
    pub keep_running: bool,
}



#[allow(unused)]
pub fn validated_cli() -> Result<ValidatedOptions, PaymentError> {
    let opt: CliOptions = CliOptions::from_args();
    match opt {
        CliOptions::Transfer(transfer_options) => {
            let split_pattern = [',', ';'];
            let mut amounts = Vec::<U256>::new();
            for amount in transfer_options.amounts.split(&split_pattern) {
                let amount = U256::from_dec_str(amount).map_err(|_| {
                    PaymentError::OtherError(format!("Invalid amount when parsing input: {amount}"))
                })?;
                amounts.push(amount);
            }

            let mut receivers = Vec::<Address>::new();
            for receiver in transfer_options.receivers.split(&split_pattern) {
                let receiver = Address::from_str(receiver).map_err(|_| {
                    PaymentError::OtherError(format!(
                        "Invalid receiver when parsing input: {receiver}"
                    ))
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
            if transfer_options.plain_eth && transfer_options.token_addr.is_some() {
                return Err(PaymentError::OtherError(format!(
                    "Can't specify both plain-eth and token-addr"
                )));
            };
            if !transfer_options.plain_eth && transfer_options.token_addr.is_none() {
                return Err(PaymentError::OtherError(format!(
                    "Specify token-addr or set plain-eth true to plain transfer"
                )));
            };

            let token_addr = if transfer_options.plain_eth {
                None
            } else {
                transfer_options
                    .token_addr
                    .map(|s| Address::from_str(&s))
                    .transpose()?
            };

            Ok(ValidatedOptions {
                receivers,
                amounts,
                chain_id: transfer_options.chain_id,
                token_addr,
                keep_running: transfer_options.keep_running,
            })
        }
        CliOptions::Test(test_options) => {
            let mut receivers = Vec::<Address>::new();
            let mut amounts = Vec::<U256>::new();
            for i in 0..test_options.generate_count {
                let gen_addr_str = &format!("0x{:040x}", i + 0x10000);
                let receiver = Address::from_str(gen_addr_str).map_err(|_| {
                    PaymentError::OtherError(format!(
                        "Invalid receiver when parsing input: {gen_addr_str}"
                    ))
                })?;
                receivers.push(receiver);
                amounts.push(U256::from(i));
            }
            Ok(ValidatedOptions {
                receivers,
                amounts,
                chain_id: test_options.chain_id,
                token_addr: None,
                keep_running: false,
            })
        }
        CliOptions::Process (process_options) => {
            //println!("magicality: {}, color: {}", magicality, color);
            Ok(ValidatedOptions {
                receivers: vec![],
                amounts: vec![],
                chain_id: 0,
                token_addr: None,
                keep_running: process_options.keep_running,
            })
        }
    }
}
