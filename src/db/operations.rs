use sqlx::SqliteConnection;

use crate::model::{Allowance, TokenTransfer, Web3TransactionDao};

pub async fn insert_token_transfer(
    conn: &mut SqliteConnection,
    token_transfer: &TokenTransfer,
) -> Result<TokenTransfer, sqlx::Error> {
    let res = sqlx::query_as::<_, TokenTransfer>(
        r"INSERT INTO token_transfer
(from_addr, receiver_addr, chain_id, token_addr, token_amount, tx_id, fee_paid, error)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING *;
",
    )
    .bind(&token_transfer.from_addr)
    .bind(&token_transfer.receiver_addr)
    .bind(&token_transfer.chain_id)
    .bind(&token_transfer.token_addr)
    .bind(&token_transfer.token_amount)
    .bind(&token_transfer.tx_id)
    .bind(&token_transfer.fee_paid)
    .bind(&token_transfer.error)
    .fetch_one(conn)
    .await?;
    Ok(res)
}

pub async fn insert_allowance(
    conn: &mut SqliteConnection,
    allowance: &Allowance,
) -> Result<Allowance, sqlx::Error> {
    let res = sqlx::query_as::<_, Allowance>(
        r"INSERT INTO allowance
(
owner,
token_addr,
spender,
allowance,
chain_id,
tx_id,
fee_paid,
confirm_date,
error
)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING *;
",
    )
    .bind(&allowance.owner)
    .bind(&allowance.token_addr)
    .bind(&allowance.spender)
    .bind(&allowance.allowance)
    .bind(&allowance.chain_id)
    .bind(&allowance.tx_id)
    .bind(&allowance.fee_paid)
    .bind(&allowance.confirm_date)
    .bind(&allowance.error)
    .fetch_one(conn)
    .await?;
    Ok(res)
}

pub async fn update_allowance(
    conn: &mut SqliteConnection,
    allowance: &Allowance,
) -> Result<Allowance, sqlx::Error> {
    let _res = sqlx::query(
        r"UPDATE allowance SET
owner = $1,
token_addr = $2,
spender = $3,
allowance = $4,
chain_id = $5,
tx_id = $6,
fee_paid = $7,
confirm_date = $8,
error = $9
 ",
    )
    .bind(&allowance.owner)
    .bind(&allowance.token_addr)
    .bind(&allowance.spender)
    .bind(&allowance.allowance)
    .bind(&allowance.chain_id)
    .bind(&allowance.tx_id)
    .bind(&allowance.fee_paid)
    .bind(&allowance.confirm_date)
    .bind(&allowance.error)
    .execute(conn)
    .await?;
    Ok(allowance.clone())
}

#[allow(unused)]
pub async fn get_all_allowances(
    conn: &mut SqliteConnection,
) -> Result<Vec<Allowance>, sqlx::Error> {
    let rows = sqlx::query_as::<_, Allowance>(r"SELECT * FROM allowance")
        .fetch_all(conn)
        .await?;
    Ok(rows)
}

pub async fn get_allowance_by_tx(
    conn: &mut SqliteConnection,
    tx_id: i64,
) -> Result<Allowance, sqlx::Error> {
    let row = sqlx::query_as::<_, Allowance>(r"SELECT * FROM allowance WHERE tx_id=$1")
        .bind(tx_id)
        .fetch_one(conn)
        .await?;
    Ok(row)
}

pub async fn find_allowance(
    conn: &mut SqliteConnection,
    owner: &str,
    token_addr: &str,
    spender: &str,
    chain_id: i64,
) -> Result<Option<Allowance>, sqlx::Error> {
    let row = sqlx::query_as::<_, Allowance>(
        r"SELECT * FROM allowance
WHERE
owner = $1 AND
token_addr = $2 AND
spender = $3 AND
chain_id = $4
",
    )
    .bind(owner)
    .bind(token_addr)
    .bind(spender)
    .bind(chain_id)
    .fetch_optional(conn)
    .await?;
    Ok(row)
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

#[allow(unused)]
pub async fn get_all_token_transfers(
    conn: &mut SqliteConnection,
) -> Result<Vec<TokenTransfer>, sqlx::Error> {
    let rows = sqlx::query_as::<_, TokenTransfer>(r"SELECT * FROM token_transfer")
        .fetch_all(conn)
        .await?;
    Ok(rows)
}

pub async fn get_pending_token_transfers(
    conn: &mut SqliteConnection,
) -> Result<Vec<TokenTransfer>, sqlx::Error> {
    let rows =
        sqlx::query_as::<_, TokenTransfer>(r"SELECT * FROM token_transfer WHERE tx_id is null")
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
(method, from_addr, to_addr, chain_id, gas_limit, max_fee_per_gas, priority_fee, val, nonce, processing, call_data, created_date, tx_hash, signed_raw_data, signed_date, broadcast_date, confirm_date, block_number, chain_status, fee_paid)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20) RETURNING *;
",
    )
        .bind(&tx.method)
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
        .bind( &tx.confirm_date)
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
method = $2,
from_addr = $3,
to_addr = $4,
chain_id = $5,
gas_limit = $6,
max_fee_per_gas = $7,
priority_fee = $8,
val = $9,
nonce = $10,
processing = $11,
call_data = $12,
created_date = $13,
tx_hash = $14,
signed_raw_data = $15,
signed_date = $16,
broadcast_date = $17,
confirm_date = $18,
block_number = $19,
chain_status = $20,
fee_paid = $21
WHERE id = $1
",
    )
    .bind(&tx.id)
    .bind(&tx.method)
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
    .bind(&tx.confirm_date)
    .bind(&tx.block_number)
    .bind(&tx.chain_status)
    .bind(&tx.fee_paid)
    .execute(conn)
    .await?;
    Ok(tx.clone())
}
