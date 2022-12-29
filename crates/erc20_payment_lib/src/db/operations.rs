use crate::model::{Allowance, ChainTransfer, TokenTransfer, Web3TransactionDao};
use sqlx::SqliteConnection;

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

pub async fn insert_chain_transfer(
    conn: &mut SqliteConnection,
    token_transfer: &ChainTransfer,
) -> Result<ChainTransfer, sqlx::Error> {
    let res = sqlx::query_as::<_, ChainTransfer>(
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
    .bind(allowance.chain_id)
    .bind(allowance.tx_id)
    .bind(&allowance.fee_paid)
    .bind(allowance.confirm_date)
    .bind(&allowance.error)
    .fetch_one(conn)
    .await?;
    Ok(res)
}

pub async fn update_allowance(
    conn: &mut SqliteConnection,
    allowance: &Allowance,
) -> Result<(), sqlx::Error> {
    let _res = sqlx::query(
        r"UPDATE allowance SET
owner = $2,
token_addr = $3,
spender = $4,
allowance = $5,
chain_id = $6,
tx_id = $7,
fee_paid = $8,
confirm_date = $9,
error = $10
WHERE id = $1
 ",
    )
    .bind(allowance.id)
    .bind(&allowance.owner)
    .bind(&allowance.token_addr)
    .bind(&allowance.spender)
    .bind(&allowance.allowance)
    .bind(allowance.chain_id)
    .bind(allowance.tx_id)
    .bind(&allowance.fee_paid)
    .bind(allowance.confirm_date)
    .bind(&allowance.error)
    .execute(conn)
    .await?;
    Ok(())
}

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

pub async fn get_allowances_by_owner(
    conn: &mut SqliteConnection,
    owner: &str,
) -> Result<Vec<Allowance>, sqlx::Error> {
    let row = sqlx::query_as::<_, Allowance>(
        r"SELECT * FROM allowance
WHERE
owner = $1
",
    )
    .bind(owner)
    .fetch_all(conn)
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
fee_paid = $8,
error = $9
WHERE id = $1
",
    )
    .bind(token_transfer.id)
    .bind(&token_transfer.from_addr)
    .bind(&token_transfer.receiver_addr)
    .bind(token_transfer.chain_id)
    .bind(&token_transfer.token_addr)
    .bind(&token_transfer.token_amount)
    .bind(token_transfer.tx_id)
    .bind(&token_transfer.fee_paid)
    .bind(&token_transfer.error)
    .execute(conn)
    .await?;
    Ok(token_transfer.clone())
}

pub async fn get_all_token_transfers(
    conn: &mut SqliteConnection,
    limit: Option<i64>,
) -> Result<Vec<TokenTransfer>, sqlx::Error> {
    let limit = limit.unwrap_or(i64::MAX);
    let rows = sqlx::query_as::<_, TokenTransfer>(
        r"SELECT * FROM token_transfer ORDER by id DESC LIMIT $1",
    )
    .bind(limit)
    .fetch_all(conn)
    .await?;
    Ok(rows)
}

pub const TRANSACTION_FILTER_QUEUED: &str = "processing > 0 AND first_processed IS NULL";
pub const TRANSACTION_FILTER_PROCESSING: &str = "processing > 0 AND first_processed IS NOT NULL";
pub const TRANSACTION_FILTER_TO_PROCESS: &str = "processing > 0";
pub const TRANSACTION_FILTER_ALL: &str = "id >= 0";
pub const TRANSACTION_FILTER_DONE: &str = "processing = 0";
pub const TRANSACTION_ORDER_BY_CREATE_DATE: &str = "created_date ASC";
pub const TRANSACTION_ORDER_BY_FIRST_PROCESSED_DATE_DESC: &str = "first_processed DESC";

pub async fn get_transactions(
    conn: &mut SqliteConnection,
    filter: Option<&str>,
    limit: Option<i64>,
    order: Option<&str>,
) -> Result<Vec<Web3TransactionDao>, sqlx::Error> {
    let limit = limit.unwrap_or(i64::MAX);
    let filter = filter.unwrap_or(TRANSACTION_FILTER_ALL);
    let order = order.unwrap_or("id DESC");
    let rows = sqlx::query_as::<_, Web3TransactionDao>(
        format!(
            r"SELECT * FROM tx WHERE {} ORDER BY {} LIMIT {}",
            filter, order, limit
        )
        .as_str(),
    )
    .fetch_all(conn)
    .await?;
    Ok(rows)
}

pub async fn get_transaction(
    conn: &mut SqliteConnection,
    tx_id: i64,
) -> Result<Web3TransactionDao, sqlx::Error> {
    let row = sqlx::query_as::<_, Web3TransactionDao>(r"SELECT * FROM tx WHERE id = $1")
        .bind(tx_id)
        .fetch_one(conn)
        .await?;
    Ok(row)
}

pub async fn get_pending_token_transfers(
    conn: &mut SqliteConnection,
) -> Result<Vec<TokenTransfer>, sqlx::Error> {
    let rows = sqlx::query_as::<_, TokenTransfer>(
        r"SELECT * FROM token_transfer
WHERE tx_id is null
AND error is null
",
    )
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

pub const TRANSFER_FILTER_ALL: &str = "(id >= 0)";
pub const TRANSFER_FILTER_QUEUED: &str = "(tx_id is null AND error is null)";
pub const TRANSFER_FILTER_PROCESSING: &str = "(tx_id is not null AND fee_paid is null)";
pub const TRANSFER_FILTER_DONE: &str = "(fee_paid is not null)";

pub async fn get_transfer_count(
    conn: &mut SqliteConnection,
    transfer_filter: Option<&str>,
    sender: Option<&str>,
    receiver: Option<&str>,
) -> Result<usize, sqlx::Error> {
    let transfer_filter = transfer_filter.unwrap_or(TRANSFER_FILTER_ALL);

    let count = if let Some(sender) = sender {
        sqlx::query_scalar::<_, i64>(
            format!(
                r"SELECT COUNT(*) FROM token_transfer WHERE {} AND from_addr = $1",
                transfer_filter
            )
            .as_str(),
        )
        .bind(sender)
        .fetch_one(conn)
        .await?
    } else if let Some(receiver) = receiver {
        sqlx::query_scalar::<_, i64>(
            format!(
                r"SELECT COUNT(*) FROM token_transfer WHERE {} AND receiver_addr = $1",
                transfer_filter
            )
            .as_str(),
        )
        .bind(receiver)
        .fetch_one(conn)
        .await?
    } else {
        sqlx::query_scalar::<_, i64>(
            format!(
                r"SELECT COUNT(*) FROM token_transfer WHERE {}",
                transfer_filter
            )
            .as_str(),
        )
        .fetch_one(conn)
        .await?
    };

    Ok(count as usize)
}

pub async fn get_transaction_count(
    conn: &mut SqliteConnection,
    transaction_filter: Option<&str>,
) -> Result<usize, sqlx::Error> {
    let transaction_filter = transaction_filter.unwrap_or(TRANSACTION_FILTER_ALL);
    let count = sqlx::query_scalar::<_, i64>(
        format!(r"SELECT COUNT(*) FROM tx WHERE {}", transaction_filter).as_str(),
    )
    .fetch_one(conn)
    .await?;
    Ok(count as usize)
}

pub async fn get_next_transactions_to_process(
    conn: &mut SqliteConnection,
    limit: i64,
) -> Result<Vec<Web3TransactionDao>, sqlx::Error> {
    get_transactions(
        conn,
        Some(TRANSACTION_FILTER_TO_PROCESS),
        Some(limit),
        Some(TRANSACTION_ORDER_BY_CREATE_DATE),
    )
    .await
}

pub async fn force_tx_error(
    conn: &mut SqliteConnection,
    tx: &Web3TransactionDao,
) -> Result<(), sqlx::Error> {
    sqlx::query(r"UPDATE tx SET error = 'forced error' WHERE id = $1")
        .bind(tx.id)
        .execute(conn)
        .await?;
    Ok(())
}

pub async fn insert_tx(
    conn: &mut SqliteConnection,
    tx: &Web3TransactionDao,
) -> Result<Web3TransactionDao, sqlx::Error> {
    let res = sqlx::query_as::<_, Web3TransactionDao>(
        r"INSERT INTO tx
(method, from_addr, to_addr, chain_id, gas_limit, max_fee_per_gas, priority_fee, val, nonce, processing, call_data, created_date, first_processed, tx_hash, signed_raw_data, signed_date, broadcast_date, broadcast_count, confirm_date, block_number, chain_status, fee_paid, error)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23) RETURNING *;
",
    )
        .bind(&tx.method)
        .bind(&tx.from_addr)
        .bind(&tx.to_addr)
        .bind( tx.chain_id)
        .bind( tx.gas_limit)
        .bind( &tx.max_fee_per_gas)
        .bind( &tx.priority_fee)
        .bind( &tx.val)
        .bind( tx.nonce)
        .bind( tx.processing)
        .bind( &tx.call_data)
        .bind( tx.created_date)
        .bind( tx.first_processed)
        .bind( &tx.tx_hash)
        .bind( &tx.signed_raw_data)
        .bind( tx.signed_date)
        .bind( tx.broadcast_date)
        .bind( tx.broadcast_count)
        .bind( tx.confirm_date)
        .bind( tx.block_number)
        .bind( tx.chain_status)
        .bind( &tx.fee_paid)
        .bind(&tx.error)
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
first_processed = $14,
tx_hash = $15,
signed_raw_data = $16,
signed_date = $17,
broadcast_date = $18,
broadcast_count = $19,
confirm_date = $20,
block_number = $21,
chain_status = $22,
fee_paid = $23,
error = $24
WHERE id = $1
",
    )
    .bind(tx.id)
    .bind(&tx.method)
    .bind(&tx.from_addr)
    .bind(&tx.to_addr)
    .bind(tx.chain_id)
    .bind(tx.gas_limit)
    .bind(&tx.max_fee_per_gas)
    .bind(&tx.priority_fee)
    .bind(&tx.val)
    .bind(tx.nonce)
    .bind(tx.processing)
    .bind(&tx.call_data)
    .bind(tx.created_date)
    .bind(tx.first_processed)
    .bind(&tx.tx_hash)
    .bind(&tx.signed_raw_data)
    .bind(tx.signed_date)
    .bind(tx.broadcast_date)
    .bind(tx.broadcast_count)
    .bind(tx.confirm_date)
    .bind(tx.block_number)
    .bind(tx.chain_status)
    .bind(&tx.fee_paid)
    .bind(&tx.error)
    .execute(conn)
    .await?;
    Ok(tx.clone())
}
