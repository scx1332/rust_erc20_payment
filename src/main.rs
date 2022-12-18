mod options;

use crate::options::validated_cli;
use actix_web::{web, App, HttpServer};
use erc20_payment_lib::db::create_sqlite_connection;
use erc20_payment_lib::server::{accounts, allowances, greet, transfers, ServerData};
use erc20_payment_lib::{
    config, err_custom_create,
    error::{CustomError, ErrorBag, PaymentError},
    misc::{display_private_keys, load_private_keys},
    runtime::start_payment_engine,
};
use std::env;
use std::sync::Arc;
use tokio::sync::Mutex;

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

    let db_conn = env::var("DB_SQLITE_FILENAME").unwrap();
    log::info!("connecting to sqlite file db: {}", db_conn);
    let mut conn = create_sqlite_connection(&db_conn, true).await?;

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
            .route("/transfers", web::get().to(transfers))
            .route("/transfers/{tx_id}", web::get().to(transfers))
            .route("/accounts", web::get().to(accounts))
            .route("/{name}", web::get().to(greet))
    })
    .bind(("127.0.0.1", 8080))
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
