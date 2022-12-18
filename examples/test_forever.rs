use erc20_payment_lib::db::create_sqlite_connection;
use erc20_payment_lib::misc::{
    create_test_amount_pool, generate_transaction_batch, ordered_address_pool,
};
use erc20_payment_lib::{
    config, err_custom_create,
    error::{CustomError, ErrorBag, PaymentError},
    misc::{display_private_keys, load_private_keys},
    runtime::start_payment_engine,
};
use std::env;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct TestOptions {
    #[structopt(long = "chain-name", default_value = "mumbai")]
    chain_name: String,

    #[structopt(long = "generate-count", default_value = "10")]
    generate_count: usize,

    #[structopt(long = "address-pool-size", default_value = "10")]
    address_pool_size: usize,

    #[structopt(long = "amounts-pool-size", default_value = "10")]
    amounts_pool_size: usize,
}

async fn main_internal() -> Result<(), PaymentError> {
    if let Err(err) = dotenv::dotenv() {
        return Err(err_custom_create!("No .env file found: {}", err));
    }
    env_logger::init();
    let cli: TestOptions = TestOptions::from_args();

    let (private_keys, public_addrs) = load_private_keys(&env::var("ETH_PRIVATE_KEYS").unwrap())?;
    display_private_keys(&private_keys);

    let config = config::Config::load("config-payments.toml")?;

    let db_conn = env::var("DB_SQLITE_FILENAME").unwrap();
    let mut conn = create_sqlite_connection(&db_conn, true).await?;

    let addr_pool = ordered_address_pool(cli.address_pool_size, false)?;
    let amount_pool = create_test_amount_pool(cli.amounts_pool_size)?;
    let c = config.chain.get(&cli.chain_name).unwrap().clone();

    let sp = start_payment_engine(None, &private_keys, config).await?;
    loop {
        {
            let idling = { sp.shared_state.lock().await.idling };
            let ignore_idling = true;
            if idling || ignore_idling {
                generate_transaction_batch(
                    &mut conn,
                    c.network_id as u64,
                    &public_addrs,
                    Some(c.token.clone().unwrap().address),
                    &addr_pool,
                    &amount_pool,
                    cli.generate_count,
                )
                .await?;
            }
            tokio::time::sleep(std::time::Duration::from_secs(10)).await;
        }
    }
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
