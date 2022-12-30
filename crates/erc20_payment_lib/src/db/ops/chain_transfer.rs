use crate::db::model::*;
use sqlx::SqliteConnection;

pub async fn insert_chain_transfer(
    conn: &mut SqliteConnection,
    token_transfer: &ChainTransferDao,
) -> Result<ChainTransferDao, sqlx::Error> {
    let res = sqlx::query_as::<_, ChainTransferDao>(
        r"INSERT INTO chain_transfer
(from_addr, receiver_addr, chain_id, token_addr, token_amount, tx_id)
VALUES ($1, $2, $3, $4, $5, $6) RETURNING *;
",
    )
    .bind(&token_transfer.from_addr)
    .bind(&token_transfer.receiver_addr)
    .bind(token_transfer.chain_id)
    .bind(&token_transfer.token_addr)
    .bind(&token_transfer.token_amount)
    .bind(token_transfer.tx_id)
    .fetch_one(conn)
    .await?;
    Ok(res)
}
