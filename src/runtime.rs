use std::env;
use secp256k1::SecretKey;
use sqlx::SqliteConnection;
use tokio::task::JoinHandle;
use crate::config;
use crate::error::PaymentError;
use crate::options::ValidatedOptions;
use crate::setup::PaymentSetup;
use crate::db::create_sqlite_connection;
use crate::db::operations::insert_token_transfer;
use crate::eth::get_eth_addr_from_secret;
use crate::service::service_loop;
use crate::transaction::create_token_transfer;
use std::str::FromStr;

async fn process_cli(
    conn: &mut SqliteConnection,
    cli: &ValidatedOptions,
    secret_key: &SecretKey,
) -> Result<(), PaymentError> {
    let from_addr = get_eth_addr_from_secret(secret_key);
    for transaction_no in 0..cli.receivers.len() {
        let receiver = cli.receivers[transaction_no];
        let amount = cli.amounts[transaction_no];
        let token_transfer = create_token_transfer(
            from_addr,
            receiver,
            cli.chain_id as u64,
            cli.token_addr,
            amount,
        );
        let _token_transfer = insert_token_transfer(conn, &token_transfer).await?;
    }
    Ok(())

    //service_loop(&mut conn, &web3, &secret_key).await;
}

pub async fn start_payment_engine(cli: Option<ValidatedOptions>) -> Result<JoinHandle<()>, PaymentError> {
    let cli = cli.unwrap_or_default();
    let config = config::Config::load("config-payments.toml")?;
    let private_key = env::var("ETH_PRIVATE_KEY").unwrap();
    let secret_key = SecretKey::from_str(&private_key).unwrap();
    let payment_setup = PaymentSetup::new(
        &config,
        secret_key,
        !cli.keep_running,
        cli.generate_tx_only,
        cli.skip_multi_contract_check,
    )?;
    log::debug!("Payment setup: {:?}", payment_setup);

    let db_conn = env::var("DB_SQLITE_FILENAME").unwrap();
    let mut conn = create_sqlite_connection(&db_conn, true).await?;

    process_cli(&mut conn, &cli, &payment_setup.secret_key).await?;

    Ok(tokio::spawn(async move { service_loop(&mut conn, payment_setup).await }))
}
