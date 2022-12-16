use crate::db::create_sqlite_connection;

use crate::error::PaymentError;

use crate::service::service_loop;
use crate::setup::PaymentSetup;

use crate::config;
use secp256k1::SecretKey;
use sqlx::SqliteConnection;
use std::env;

use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use web3::types::{Address, U256};

pub struct SharedState {
    pub inserted: usize,
    pub idling: bool,
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
    pub service_sleep: u64,
    pub process_sleep: u64,
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
            service_sleep: 10,
            process_sleep: 10,
        }
    }
}
pub struct PaymentRuntime {
    pub runtime_handle: JoinHandle<()>,
    pub setup: PaymentSetup,
    pub shared_state: Arc<Mutex<SharedState>>,
    pub conn: Arc<Mutex<SqliteConnection>>,
}

/*
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
        let _token_transfer = insert_token_transfer(conn, &token_transfer)
            .await
            .map_err(err_from!())?;
    }
    Ok(())

    //service_loop(&mut conn, &web3, &secret_key).await;
}
*/

pub async fn start_payment_engine(
    cli: Option<ValidatedOptions>,
    secret_keys: &[SecretKey],
    config: config::Config,
) -> Result<PaymentRuntime, PaymentError> {
    let cli = cli.unwrap_or_default();
    let payment_setup = PaymentSetup::new(
        &config,
        secret_keys.to_vec(),
        !cli.keep_running,
        cli.generate_tx_only,
        cli.skip_multi_contract_check,
        cli.service_sleep,
        cli.process_sleep,
    )?;
    log::debug!("Starting payment engine: {:#?}", payment_setup);

    let db_conn = env::var("DB_SQLITE_FILENAME").unwrap();
    log::info!("connecting to sqlite file db: {}", db_conn);
    let mut conn = create_sqlite_connection(&db_conn, true).await?;
    let conn2 = create_sqlite_connection(&db_conn, false).await?;

    //process_cli(&mut conn, &cli, &payment_setup.secret_key).await?;

    let ps = payment_setup.clone();

    let shared_state = Arc::new(Mutex::new(SharedState { inserted: 0, idling: false }));
    let shared_state_clone = shared_state.clone();
    let jh = tokio::spawn(async move { service_loop(shared_state_clone, &mut conn, &ps).await });

    Ok(PaymentRuntime {
        runtime_handle: jh,
        setup: payment_setup,
        shared_state,
        conn: Arc::new(Mutex::new(conn2)),
    })
}
