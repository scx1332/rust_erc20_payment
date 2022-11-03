
use sqlx::SqliteConnection;
use std::error::Error;

use crate::model::{TokenTransfer, Web3TransactionDao};

pub async fn insert_token_transfer(
    conn: &mut SqliteConnection,
    token_transfer: &TokenTransfer,
) -> Result<TokenTransfer, sqlx::Error> {
    let res = sqlx::query_as::<_, TokenTransfer>(
        r"INSERT INTO token_transfer
(from_addr, receiver_addr, chain_id, token_addr, token_amount, tx_id, fee_paid)
VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING *;
",
    )
    .bind(&token_transfer.from_addr)
    .bind(&token_transfer.receiver_addr)
    .bind(&token_transfer.chain_id)
    .bind(&token_transfer.token_addr)
    .bind(&token_transfer.token_amount)
    .bind(&token_transfer.tx_id)
    .bind(&token_transfer.fee_paid)
    .fetch_one(conn)
    .await?;
    Ok(res)
}

pub async fn update_token_transfer(
    conn: &mut SqliteConnection,
    token_transfer: &TokenTransfer,
) -> Result<TokenTransfer, sqlx::Error> {
    let _res = sqlx::query(
        r"UPDATE token_transfer SET
from_addr = $2,
receiver_addr = $3,
chain_id = $4,
token_addr = $5,
token_amount = $6,
tx_id = $7,
fee_paid = $8
WHERE id = $1
",
    )
    .bind(&token_transfer.id)
    .bind(&token_transfer.from_addr)
    .bind(&token_transfer.receiver_addr)
    .bind(&token_transfer.chain_id)
    .bind(&token_transfer.token_addr)
    .bind(&token_transfer.token_amount)
    .bind(&token_transfer.tx_id)
    .bind(&token_transfer.fee_paid)
    .execute(conn)
    .await?;
    Ok(token_transfer.clone())
}

pub async fn get_all_token_transfers(
    conn: &mut SqliteConnection,
) -> Result<Vec<TokenTransfer>, sqlx::Error> {
    let rows = sqlx::query_as::<_, TokenTransfer>(r"SELECT * FROM token_transfer")
        .fetch_all(conn)
        .await?;
    Ok(rows)
}

pub async fn get_token_transfers_by_tx(
    conn: &mut SqliteConnection,
    tx_id: i64,
) -> Result<Vec<TokenTransfer>, sqlx::Error> {
    let rows = sqlx::query_as::<_, TokenTransfer>(r"SELECT * FROM token_transfer WHERE tx_id=$1")
        .bind(tx_id)
        .fetch_all(conn)
        .await?;
    Ok(rows)
}

pub async fn get_transactions_being_processed(
    conn: &mut SqliteConnection,
) -> Result<Vec<Web3TransactionDao>, sqlx::Error> {
    let rows = sqlx::query_as::<_, Web3TransactionDao>(
        r"SELECT * FROM tx WHERE processing>0 ORDER by nonce DESC",
    )
    .fetch_all(conn)
    .await?;
    Ok(rows)
}

pub async fn insert_tx(
    conn: &mut SqliteConnection,
    tx: &Web3TransactionDao,
) -> Result<Web3TransactionDao, sqlx::Error> {
    let res = sqlx::query_as::<_, Web3TransactionDao>(
        r"INSERT INTO tx
(from_addr, to_addr, chain_id, gas_limit, max_fee_per_gas, priority_fee, val, nonce, processing, call_data, created_date, tx_hash, signed_raw_data, signed_date, broadcast_date, confirmed_date, block_number, chain_status, fee_paid)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19) RETURNING *;
",
    )
        .bind(&tx.from_addr)
        .bind(&tx.to_addr)
        .bind( &tx.chain_id)
        .bind( &tx.gas_limit)
        .bind( &tx.max_fee_per_gas)
        .bind( &tx.priority_fee)
        .bind( &tx.val)
        .bind( &tx.nonce)
        .bind( &tx.processing)
        .bind( &tx.call_data)
        .bind( &tx.created_date)
        .bind( &tx.tx_hash)
        .bind( &tx.signed_raw_data)
        .bind( &tx.signed_date)
        .bind( &tx.broadcast_date)
        .bind( &tx.confirmed_date)
        .bind( &tx.block_number)
        .bind( &tx.chain_status)
        .bind( &tx.fee_paid)
        .fetch_one(conn)
        .await?;
    Ok(res)
}

pub async fn update_tx(
    conn: &mut SqliteConnection,
    tx: &Web3TransactionDao,
) -> Result<Web3TransactionDao, sqlx::Error> {
    let _res = sqlx::query(
        r"UPDATE tx SET
from_addr = $2,
to_addr = $3,
chain_id = $4,
gas_limit = $5,
max_fee_per_gas = $6,
priority_fee = $7,
val = $8,
nonce = $9,
processing = $10,
call_data = $11,
created_date = $12,
tx_hash = $13,
signed_raw_data = $14,
signed_date = $15,
broadcast_date = $16,
confirmed_date = $17,
block_number = $18,
chain_status = $19,
fee_paid = $20
WHERE id = $1
",
    )
    .bind(&tx.id)
    .bind(&tx.from_addr)
    .bind(&tx.to_addr)
    .bind(&tx.chain_id)
    .bind(&tx.gas_limit)
    .bind(&tx.max_fee_per_gas)
    .bind(&tx.priority_fee)
    .bind(&tx.val)
    .bind(&tx.nonce)
    .bind(&tx.processing)
    .bind(&tx.call_data)
    .bind(&tx.created_date)
    .bind(&tx.tx_hash)
    .bind(&tx.signed_raw_data)
    .bind(&tx.signed_date)
    .bind(&tx.broadcast_date)
    .bind(&tx.confirmed_date)
    .bind(&tx.block_number)
    .bind(&tx.chain_status)
    .bind(&tx.fee_paid)
    .execute(conn)
    .await?;
    Ok(tx.clone())
}
