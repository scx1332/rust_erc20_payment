use sqlx::postgres::PgPoolOptions;
use sqlx::{QueryBuilder, Sqlite, SqliteConnection, SqlitePool};
use sqlx::{Pool, Postgres};
use std::env;
use std::error::Error;
use std::str::FromStr;
use sqlx::migrate::Migrator;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::ConnectOptions;

static MIGRATOR: Migrator = sqlx::migrate!();

pub async fn create_postgres_connection(max_connections: u32) -> Result<Pool<Postgres>, Box<dyn Error>> {
    let db_type = env::var("DB_TYPE").unwrap_or_else(|_| "sqlite".to_string());

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

pub async fn create_sqlite_connection(max_connections: u32) -> Result<SqliteConnection, Box<dyn Error>> {
    let url = format!("sqlite://db.sqlite");

    log::info!("connecting to db using url {}", url);

    let mut conn = SqliteConnectOptions::from_str("sqlite://db.sqlite")?.create_if_missing(true).connect().await?;

    MIGRATOR.run(&mut conn).await?;

    Ok(conn)
}
