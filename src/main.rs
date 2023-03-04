mod options;
use crate::options::CliOptions;
use actix_web::Scope;
use actix_web::{web, App, HttpServer};
use erc20_payment_lib::config::AdditionalOptions;
use erc20_payment_lib::db::create_sqlite_connection;
use erc20_payment_lib::misc::load_public_addresses;
use erc20_payment_lib::server::*;
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

async fn main_internal() -> Result<(), PaymentError> {
    if let Err(err) = dotenv::dotenv() {
        return Err(err_custom_create!("No .env file found: {}", err));
    }
    env_logger::init();
    let cli: CliOptions = CliOptions::from_args();

    let (private_keys, _public_addrs) = load_private_keys(
        &env::var("ETH_PRIVATE_KEYS").expect("Specify ETH_PRIVATE_KEYS env variable"),
    )?;
    let receiver_accounts = load_public_addresses(
        &env::var("ETH_RECEIVERS").expect("Specify ETH_RECEIVERS env variable"),
    )?;
    display_private_keys(&private_keys);

    let config = config::Config::load("config-payments.toml")?;

    if cli.http && !cli.keep_running {
        return Err(err_custom_create!("http mode requires keep-running option"));
    }

    let add_opt = AdditionalOptions {
        keep_running: cli.keep_running,
        generate_tx_only: cli.generate_tx_only,
        skip_multi_contract_check: cli.skip_multi_contract_check,
    };
    let db_filename = env::var("DB_SQLITE_FILENAME").unwrap();
    log::info!("connecting to sqlite file db: {}", db_filename);
    let conn = create_sqlite_connection(Some(&db_filename), true).await?;
    let sp = start_payment_engine(
        &private_keys,
        &receiver_accounts,
        &db_filename,
        config,
        Some(conn.clone()),
        Some(add_opt),
    )
    .await?;

    let server_data = web::Data::new(Box::new(ServerData {
        shared_state: sp.shared_state.clone(),
        db_connection: Arc::new(Mutex::new(conn)),
        payment_setup: sp.setup.clone(),
    }));

    if cli.http {
        let server = HttpServer::new(move || {
            let cors = actix_cors::Cors::default()
                .allow_any_origin()
                .allow_any_method()
                .allow_any_header()
                .max_age(3600);

            let scope = runtime_web_scope(
                Scope::new("erc20"),
                server_data.clone(),
                cli.faucet,
                cli.debug,
                cli.faucet,
            );

            App::new().wrap(cors).service(scope)
        })
        .workers(cli.http_threads as usize)
        .bind((cli.http_addr.as_str(), cli.http_port))
        .expect("Cannot run server")
        .run();

        log::info!(
            "http server starting on {}:{}",
            cli.http_addr,
            cli.http_port
        );

        server.await.unwrap();
    } else {
        sp.runtime_handle.await.unwrap();
    }

    Ok(())
}

#[actix_web::main]
async fn main() -> Result<(), PaymentError> {
    match main_internal().await {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("Error: {e}");
            Err(e)
        }
    }
}
