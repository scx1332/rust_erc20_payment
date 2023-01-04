use crate::err_from;
use crate::error::PaymentError;
use crate::error::*;
use sqlx::migrate::Migrator;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::ConnectOptions;
use sqlx::SqliteConnection;
use std::str::FromStr;

static MIGRATOR: Migrator = sqlx::migrate!();

pub async fn create_sqlite_connection(
    file_name: Option<&str>,
    run_migrations: bool,
) -> Result<SqliteConnection, PaymentError> {
    let url = if let Some(file_name) = file_name {
        format!("sqlite://{}", file_name)
    } else {
        "sqlite::memory:".to_string()
    };

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
