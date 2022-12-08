use crate::err_from;
use crate::error::PaymentError;
use crate::error::*;
use sqlx::migrate::Migrator;
use sqlx::postgres::PgPoolOptions;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::ConnectOptions;
use sqlx::SqliteConnection;
use sqlx::{Pool, Postgres};
use std::env;
use std::error::Error;
use std::str::FromStr;

static MIGRATOR: Migrator = sqlx::migrate!();

pub async fn _create_postgres_connection(
    max_connections: u32,
) -> Result<Pool<Postgres>, Box<dyn Error>> {
    let _db_type = env::var("DB_TYPE").unwrap_or_else(|_| "sqlite".to_string());

    let username = env::var("DB_USER").unwrap_or_else(|_| String::from("postgres"));
    let password = env::var("DB_PASS").unwrap_or_else(|_| String::from("postgres"));
    let db_addr = env::var("DB_ADDR").unwrap_or_else(|_| String::from("db"));

    let url = format!(
        "postgres://{user}:{pass}@{addr}/GolemHub",
        user = username,
        pass = password,
        addr = db_addr
    );

    log::info!("connecting to db using url {}", url);

    let pool = PgPoolOptions::new()
        .max_connections(max_connections)
        .connect_lazy(url.as_str())?;

    MIGRATOR.run(&pool).await?;

    Ok(pool)
}

pub async fn create_sqlite_connection(
    file_name: &str,
    run_migrations: bool,
) -> Result<SqliteConnection, PaymentError> {
    let url = format!("sqlite://{}", file_name);

    log::info!("connecting to db using url {}", url);

    let mut conn = SqliteConnectOptions::from_str(&url)
        .map_err(err_from!())?
        .create_if_missing(true)
        .connect()
        .await
        .map_err(err_from!())?;

    if run_migrations {
        MIGRATOR.run(&mut conn).await.map_err(err_from!())?;
    }

    Ok(conn)
}