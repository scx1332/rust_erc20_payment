use crate::error::CustomError;
use crate::error::ErrorBag;
use crate::error::PaymentError;
use crate::{err_custom_create, err_from, err_from_msg};
use std::fmt::Debug;
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

    #[structopt(
        long = "generate-tx-only",
        help = "Do not send or process transactions, only generate stubs"
    )]
    generate_tx_only: bool,

    #[structopt(
        long = "skip-multi-contract-check",
        help = "Skip multi contract check when generating txs"
    )]
    skip_multi_contract_check: bool,
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

    #[structopt(
        long = "generate-tx-only",
        help = "Do not send or process transactions, only generate stubs"
    )]
    generate_tx_only: bool,

    #[structopt(
        long = "skip-multi-contract-check",
        help = "Skip multi contract check when generating txs"
    )]
    skip_multi_contract_check: bool,
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
    pub generate_tx_only: bool,
    pub skip_multi_contract_check: bool,
}

impl Default for ValidatedOptions {
    fn default() -> Self {
        ValidatedOptions {
            receivers: vec![],
            amounts: vec![],
            chain_id: 80001,
            token_addr: None,
            keep_running: true,
            generate_tx_only: false,
            skip_multi_contract_check: false,
        }
    }
}

#[allow(unused)]
pub fn validated_cli() -> Result<ValidatedOptions, PaymentError> {
    let opt: CliOptions = CliOptions::from_args();
    match opt {
        CliOptions::Transfer(transfer_options) => {
            let split_pattern = [',', ';'];
            let mut amounts = Vec::<U256>::new();
            for amount in transfer_options.amounts.split(&split_pattern) {
                let amount = U256::from_dec_str(amount)
                    .map_err(err_from_msg!("Invalid amount when parsing input: {amount}"))?;
                amounts.push(amount);
            }

            let mut receivers = Vec::<Address>::new();
            for receiver in transfer_options.receivers.split(&split_pattern) {
                let receiver = Address::from_str(receiver).map_err(err_from_msg!(
                    "Invalid receiver when parsing input: {receiver}"
                ))?;
                receivers.push(receiver);
            }

            if receivers.len() != amounts.len() {
                return Err(err_custom_create!(
                    "Receivers count and amount count don't match: {} != {}",
                    receivers.len(),
                    amounts.len()
                ));
            };
            if receivers.is_empty() {
                return Err(err_custom_create!("No receivers specified"));
            };
            if transfer_options.plain_eth && transfer_options.token_addr.is_some() {
                return Err(err_custom_create!(
                    "Can't specify both plain-eth and token-addr",
                ));
            };
            if !transfer_options.plain_eth && transfer_options.token_addr.is_none() {
                return Err(err_custom_create!(
                    "Specify token-addr or set plain-eth true to plain transfer",
                ));
            };

            let token_addr = if transfer_options.plain_eth {
                None
            } else {
                transfer_options
                    .token_addr
                    .map(|s| Address::from_str(&s))
                    .transpose()
                    .map_err(err_from!())?
            };

            Ok(ValidatedOptions {
                receivers,
                amounts,
                chain_id: transfer_options.chain_id,
                token_addr,
                keep_running: transfer_options.keep_running,
                generate_tx_only: transfer_options.generate_tx_only,
                skip_multi_contract_check: transfer_options.skip_multi_contract_check,
            })
        }
        CliOptions::Test(test_options) => {
            let mut receivers = Vec::<Address>::new();
            let mut amounts = Vec::<U256>::new();
            for i in 0..test_options.generate_count {
                let gen_addr_str = &format!("0x{:040x}", i + 0x10000);
                let receiver = Address::from_str(gen_addr_str).map_err(|_| {
                    err_custom_create!("Invalid receiver when parsing input: {gen_addr_str}")
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
                generate_tx_only: false,
                skip_multi_contract_check: false,
            })
        }
        CliOptions::Process(process_options) => Ok(ValidatedOptions {
            receivers: vec![],
            amounts: vec![],
            chain_id: 0,
            token_addr: None,
            keep_running: process_options.keep_running,
            generate_tx_only: process_options.generate_tx_only,
            skip_multi_contract_check: process_options.skip_multi_contract_check,
        }),
    }
}
