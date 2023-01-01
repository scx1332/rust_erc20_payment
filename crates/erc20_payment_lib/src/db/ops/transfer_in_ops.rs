use crate::db::model::*;
use sqlx::SqliteConnection;

pub async fn insert_transfer_in(
    conn: &mut SqliteConnection,
    token_transfer: &TransferInDao,
) -> Result<TokenTransferDao, sqlx::Error> {
    let res = sqlx::query_as::<_, TokenTransferDao>(
        r"INSERT INTO transfer_in
(from_addr, receiver_addr, chain_id, token_addr, token_amount, tx_hash, received_date)
VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING *;
",
    )
        .bind(&token_transfer.from_addr)
        .bind(&token_transfer.receiver_addr)
        .bind(token_transfer.chain_id)
        .bind(&token_transfer.token_addr)
        .bind(&token_transfer.token_amount)
        .bind(&token_transfer.tx_hash)
        .bind(token_transfer.received_date)
        .fetch_one(conn)
        .await?;
    Ok(res)
}

pub async fn update_transfer_in(
    conn: &mut SqliteConnection,
    token_transfer: &TransferInDao,
) -> Result<TransferInDao, sqlx::Error> {
    let _res = sqlx::query(
        r"UPDATE token_transfer SET
from_addr = $2,
receiver_addr = $3,
chain_id = $4,
token_addr = $5,
token_amount = $6,
tx_hash = $7,
received_date = $8,
WHERE id = $1
",
    )
        .bind(token_transfer.id)
        .bind(&token_transfer.from_addr)
        .bind(&token_transfer.receiver_addr)
        .bind(token_transfer.chain_id)
        .bind(&token_transfer.token_addr)
        .bind(&token_transfer.token_amount)
        .bind(&token_transfer.tx_hash)
        .bind(token_transfer.received_date)
        .execute(conn)
        .await?;
    Ok(token_transfer.clone())
}

pub async fn get_all_transfers_in(
    conn: &mut SqliteConnection,
    limit: Option<i64>,
) -> Result<Vec<TransferInDao>, sqlx::Error> {
    let limit = limit.unwrap_or(i64::MAX);
    let rows = sqlx::query_as::<_, TransferInDao>(
        r"SELECT * FROM token_transfer ORDER by id DESC LIMIT $1",
    )
        .bind(limit)
        .fetch_all(conn)
        .await?;
    Ok(rows)
}

