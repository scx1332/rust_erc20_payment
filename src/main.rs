mod options;

use crate::options::validated_cli;
use erc20_payment_lib::{
    config, err_custom_create,
    error::{CustomError, ErrorBag, PaymentError},
    misc::{display_private_keys, load_private_keys},
    runtime::start_payment_engine,
};
use std::env;

async fn main_internal() -> Result<(), PaymentError> {
    if let Err(err) = dotenv::dotenv() {
        return Err(err_custom_create!("No .env file found: {}", err));
    }
    env_logger::init();
    let cli = validated_cli()?;

    let (private_keys, _public_addrs) = load_private_keys(
        &env::var("ETH_PRIVATE_KEYS").expect("Specify ETH_PRIVATE_KEYS env variable"),
    )?;
    display_private_keys(&private_keys);

    let config = config::Config::load("config-payments.toml")?;

    let sp = start_payment_engine(Some(cli), &private_keys, config).await?;
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
