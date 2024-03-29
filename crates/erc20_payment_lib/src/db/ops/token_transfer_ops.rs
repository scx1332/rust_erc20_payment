use crate::db::model::*;
use sqlx::SqlitePool;
use sqlx_core::executor::Executor;
use sqlx_core::sqlite::Sqlite;

pub async fn insert_token_transfer(
    conn: &SqlitePool,
    token_transfer: &TokenTransferDao,
) -> Result<TokenTransferDao, sqlx::Error> {
    let res = sqlx::query_as::<_, TokenTransferDao>(
        r"INSERT INTO token_transfer
(payment_id, from_addr, receiver_addr, chain_id, token_addr, token_amount, tx_id, fee_paid, error)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING *;
",
    )
    .bind(&token_transfer.payment_id)
    .bind(&token_transfer.from_addr)
    .bind(&token_transfer.receiver_addr)
    .bind(token_transfer.chain_id)
    .bind(&token_transfer.token_addr)
    .bind(&token_transfer.token_amount)
    .bind(token_transfer.tx_id)
    .bind(&token_transfer.fee_paid)
    .bind(&token_transfer.error)
    .fetch_one(conn)
    .await?;
    Ok(res)
}

pub async fn update_token_transfer<'c, E>(
    executor: E,
    token_transfer: &TokenTransferDao,
) -> Result<TokenTransferDao, sqlx::Error>
where
    E: Executor<'c, Database = Sqlite>,
{
    let _res = sqlx::query(
        r"UPDATE token_transfer SET
payment_id = $2,
from_addr = $3,
receiver_addr = $4,
chain_id = $5,
token_addr = $6,
token_amount = $7,
tx_id = $8,
fee_paid = $9,
error = $10
WHERE id = $1
",
    )
    .bind(token_transfer.id)
    .bind(&token_transfer.payment_id)
    .bind(&token_transfer.from_addr)
    .bind(&token_transfer.receiver_addr)
    .bind(token_transfer.chain_id)
    .bind(&token_transfer.token_addr)
    .bind(&token_transfer.token_amount)
    .bind(token_transfer.tx_id)
    .bind(&token_transfer.fee_paid)
    .bind(&token_transfer.error)
    .execute(executor)
    .await?;
    Ok(token_transfer.clone())
}

pub async fn get_all_token_transfers(
    conn: &SqlitePool,
    limit: Option<i64>,
) -> Result<Vec<TokenTransferDao>, sqlx::Error> {
    let limit = limit.unwrap_or(i64::MAX);
    let rows = sqlx::query_as::<_, TokenTransferDao>(
        r"SELECT * FROM token_transfer ORDER by id DESC LIMIT $1",
    )
    .bind(limit)
    .fetch_all(conn)
    .await?;
    Ok(rows)
}

pub async fn get_pending_token_transfers(
    conn: &SqlitePool,
) -> Result<Vec<TokenTransferDao>, sqlx::Error> {
    let rows = sqlx::query_as::<_, TokenTransferDao>(
        r"SELECT * FROM token_transfer
WHERE tx_id is null
AND error is null
",
    )
    .fetch_all(conn)
    .await?;
    Ok(rows)
}

pub async fn get_token_transfers_by_tx<'c, E>(
    executor: E,
    tx_id: i64,
) -> Result<Vec<TokenTransferDao>, sqlx::Error>
where
    E: Executor<'c, Database = Sqlite>,
{
    let rows =
        sqlx::query_as::<_, TokenTransferDao>(r"SELECT * FROM token_transfer WHERE tx_id=$1")
            .bind(tx_id)
            .fetch_all(executor)
            .await?;
    Ok(rows)
}

pub const TRANSFER_FILTER_ALL: &str = "(id >= 0)";
pub const TRANSFER_FILTER_QUEUED: &str = "(tx_id is null AND error is null)";
pub const TRANSFER_FILTER_PROCESSING: &str = "(tx_id is not null AND fee_paid is null)";
pub const TRANSFER_FILTER_DONE: &str = "(fee_paid is not null)";

pub async fn get_transfer_count(
    conn: &SqlitePool,
    transfer_filter: Option<&str>,
    sender: Option<&str>,
    receiver: Option<&str>,
) -> Result<usize, sqlx::Error> {
    let transfer_filter = transfer_filter.unwrap_or(TRANSFER_FILTER_ALL);

    let count = if let Some(sender) = sender {
        sqlx::query_scalar::<_, i64>(
            format!(
                r"SELECT COUNT(*) FROM token_transfer WHERE {transfer_filter} AND from_addr = $1"
            )
            .as_str(),
        )
        .bind(sender)
        .fetch_one(conn)
        .await?
    } else if let Some(receiver) = receiver {
        sqlx::query_scalar::<_, i64>(
            format!(
                r"SELECT COUNT(*) FROM token_transfer WHERE {transfer_filter} AND receiver_addr = $1"
            )
            .as_str(),
        )
        .bind(receiver)
        .fetch_one(conn)
        .await?
    } else {
        sqlx::query_scalar::<_, i64>(
            format!(r"SELECT COUNT(*) FROM token_transfer WHERE {transfer_filter}").as_str(),
        )
        .fetch_one(conn)
        .await?
    };

    Ok(count as usize)
}
