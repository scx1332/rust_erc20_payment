use crate::config;
use crate::db::create_sqlite_connection;
use crate::db::operations::insert_token_transfer;
use crate::error::PaymentError;
use crate::eth::get_eth_addr_from_secret;
use crate::options::ValidatedOptions;
use crate::service::service_loop;
use crate::setup::PaymentSetup;
use crate::transaction::create_token_transfer;
use secp256k1::SecretKey;
use sqlx::SqliteConnection;
use std::env;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

pub struct SharedState {
    pub inserted: usize,
}

pub struct PaymentRuntime {
    pub runtime_handle: JoinHandle<()>,
    pub setup: PaymentSetup,
    pub shared_state: Arc<Mutex<SharedState>>,
    pub conn: Arc<Mutex<SqliteConnection>>,
}

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

pub async fn start_payment_engine(
    cli: Option<ValidatedOptions>,
    secret_key: &SecretKey,
    config: config::Config,
) -> Result<PaymentRuntime, PaymentError> {
    let cli = cli.unwrap_or_default();
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
    let mut conn2 = create_sqlite_connection(&db_conn, false).await?;

    process_cli(&mut conn, &cli, &payment_setup.secret_key).await?;

    let ps = payment_setup.clone();

    let shared_state = Arc::new(Mutex::new(SharedState { inserted: 0 }));

    let jh = tokio::spawn(async move {
        service_loop(&mut conn, &ps).await
    });

    Ok(PaymentRuntime {
        runtime_handle: jh,
        setup: payment_setup,
        shared_state,
        conn: Arc::new(Mutex::new(conn2)),
    })

}
