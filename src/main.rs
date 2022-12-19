mod options;

use crate::options::CliOptions;
use actix_web::{web, App, HttpServer};
use erc20_payment_lib::config::AdditionalOptions;
use erc20_payment_lib::db::create_sqlite_connection;
use erc20_payment_lib::server::{
    accounts, allowances, greet, transactions, transfers, tx_details, ServerData,
};
use erc20_payment_lib::{
    config, err_custom_create,
    error::{CustomError, ErrorBag, PaymentError},
    misc::{display_private_keys, load_private_keys},
    runtime::start_payment_engine,
};
use std::env;
use std::sync::Arc;
use structopt::StructOpt;
use tokio::sync::Mutex;
use web3::ethabi::Token::Address;

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
    let sp = start_payment_engine(&private_keys, config, Some(add_opt)).await?;

    let db_conn = env::var("DB_SQLITE_FILENAME").unwrap();
    log::info!("connecting to sqlite file db: {}", db_conn);
    let conn = create_sqlite_connection(&db_conn, true).await?;

    let server_data = web::Data::new(Box::new(ServerData {
        shared_state: sp.shared_state.clone(),
        db_connection: Arc::new(Mutex::new(conn)),
        payment_setup: sp.setup.clone(),
    }));

    let server = HttpServer::new(move || {
        App::new()
            .app_data(server_data.clone())
            .route("/", web::get().to(greet))
            .route("/allowances", web::get().to(allowances))
            .route("/transactions", web::get().to(transactions))
            .route("/tx/{tx_id}", web::get().to(tx_details))
            .route("/transfers", web::get().to(transfers))
            .route("/transfers/{tx_id}", web::get().to(transfers))
            .route("/accounts", web::get().to(accounts))
            .route("/{name}", web::get().to(greet))
    })
    .workers(cli.http_threads as usize)
    .bind((cli.http_addr.as_str(), cli.http_port))
    .expect("Cannot run server")
    .run();

    server.await.unwrap();
    Ok(())
}

#[actix_web::main]
async fn main() -> Result<(), PaymentError> {
    match main_internal().await {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("Error: {}", e);
            Err(e)
        }
    }
}
