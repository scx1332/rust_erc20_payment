use std::error::Error;
use sqlx::SqliteConnection;
use web3::types::Res;
use crate::model::TokenTransfer;


pub async fn insert_token_transfer(conn: &mut SqliteConnection, token_transfer: &TokenTransfer) -> Result<TokenTransfer, Box<dyn Error>>{

    let token_transfer = sqlx::query_as::<_, TokenTransfer>(
        r"INSERT INTO token_transfer
(id, from_addr, receiver_addr, chain_id, token_addr, token_amount, tx_id, fee_paid)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
RETURNING *",
    )
        .bind(&token_transfer.id)
        .bind(&token_transfer.from_addr)
        .bind(&token_transfer.receiver_addr)
        .bind( &token_transfer.chain_id)
        .bind( &token_transfer.token_addr)
        .bind( &token_transfer.token_amount)
        .bind( &token_transfer.tx_id)
        .bind( &token_transfer.fee_paid)
        .fetch_one(conn)
        .await?;
    Ok(token_transfer)
}