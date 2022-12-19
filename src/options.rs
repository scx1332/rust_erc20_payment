use erc20_payment_lib::error::CustomError;
use erc20_payment_lib::error::ErrorBag;
use erc20_payment_lib::error::PaymentError;
use erc20_payment_lib::runtime::ValidatedOptions;
use erc20_payment_lib::{err_custom_create, err_from, err_from_msg};
use std::fmt::Debug;
use std::str::FromStr;
use structopt::StructOpt;
use web3::types::{Address, U256};

#[derive(Debug, StructOpt)]
pub struct CliOptions {
    #[structopt(
        long = "keep-running",
        help = "Set to keep running when finished processing transactions"
    )]
    pub keep_running: bool,

    #[structopt(
        long = "generate-tx-only",
        help = "Do not send or process transactions, only generate stubs"
    )]
    pub generate_tx_only: bool,

    #[structopt(
        long = "skip-multi-contract-check",
        help = "Skip multi contract check when generating txs"
    )]
    pub skip_multi_contract_check: bool,

    #[structopt(
        long = "service-sleep",
        help = "Sleep time between service loops in seconds",
        default_value = "10"
    )]
    pub service_sleep: u64,

    #[structopt(
        long = "process-sleep",
        help = "Sleep time between process loops in seconds",
        default_value = "10"
    )]
    pub process_sleep: u64,
}
