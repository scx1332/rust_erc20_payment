mod options;

use erc20_payment_lib::{
    config, err_custom_create,
    error::{CustomError, ErrorBag, PaymentError},
    misc::{display_private_keys, load_private_keys},
    runtime::start_payment_engine,
};
use std::env;
use crate::options::CliOptions;
use structopt::StructOpt;
use web3::ethabi::Token::Address;
use erc20_payment_lib::config::AdditionalOptions;

async fn main_internal() -> Result<(), PaymentError> {
    if let Err(err) = dotenv::dotenv() {
        return Err(err_custom_create!("No .env file found: {}", err));
    }
    env_logger::init();
    let cli = CliOptions::from_args();

    let (private_keys, _public_addrs) = load_private_keys(
        &env::var("ETH_PRIVATE_KEYS").expect("Specify ETH_PRIVATE_KEYS env variable"),
    )?;
    display_private_keys(&private_keys);

    let config = config::Config::load("config-payments.toml")?;

    let add_opt = AdditionalOptions {
        keep_running: cli.keep_running,
        generate_tx_only: cli.generate_tx_only,
        skip_multi_contract_check: cli.skip_multi_contract_check,
    };

    let sp = start_payment_engine(&private_keys, config,Some(add_opt)).await?;
    sp.runtime_handle
        .await
        .map_err(|e| err_custom_create!("Service loop failed: {:?}", e))?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), PaymentError> {
    match main_internal().await {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("Error: {}", e);
            Err(e)
        }
    }
}
