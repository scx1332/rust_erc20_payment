use crate::db::model::*;
use sqlx::SqliteConnection;

pub async fn insert_chain_transfer(
    conn: &mut SqliteConnection,
    chain_transfer: &ChainTransferDao,
) -> Result<ChainTransferDao, sqlx::Error> {
    let res = sqlx::query_as::<_, ChainTransferDao>(
        r"INSERT INTO chain_transfer
(from_addr, receiver_addr, chain_id, token_addr, token_amount, chain_tx_id)
VALUES ($1, $2, $3, $4, $5, $6) RETURNING *;
",
    )
    .bind(&chain_transfer.from_addr)
    .bind(&chain_transfer.receiver_addr)
    .bind(chain_transfer.chain_id)
    .bind(&chain_transfer.token_addr)
    .bind(&chain_transfer.token_amount)
    .bind(chain_transfer.chain_tx_id)
    .fetch_one(conn)
    .await?;
    Ok(res)
}
