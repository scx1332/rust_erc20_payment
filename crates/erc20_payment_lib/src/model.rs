use chrono::{DateTime, Utc};

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct Allowance {
    pub id: i64,
    pub owner: String,
    pub token_addr: String,
    pub spender: String,
    pub allowance: String,
    pub chain_id: i64,
    pub tx_id: Option<i64>,
    pub fee_paid: Option<String>,
    pub confirm_date: Option<DateTime<Utc>>,
    pub error: Option<String>,
}

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct TokenTransfer {
    pub id: i64,
    pub from_addr: String,
    pub receiver_addr: String,
    pub chain_id: i64,
    pub token_addr: Option<String>,
    pub token_amount: String,
    pub tx_id: Option<i64>,
    pub fee_paid: Option<String>,
    pub error: Option<String>,
}

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct Web3TransactionDao {
    pub id: i64,
    pub method: String,
    pub from_addr: String,
    pub to_addr: String,
    pub chain_id: i64,
    pub gas_limit: Option<i64>,
    pub max_fee_per_gas: String,
    pub priority_fee: String,
    pub val: String,
    pub nonce: Option<i64>,
    pub processing: i64,
    pub call_data: Option<String>,
    pub created_date: DateTime<Utc>,
    pub first_processed: Option<DateTime<Utc>>,
    pub tx_hash: Option<String>,
    pub signed_raw_data: Option<String>,
    pub signed_date: Option<DateTime<Utc>>,
    pub broadcast_date: Option<DateTime<Utc>>,
    pub broadcast_count: i64,
    pub confirm_date: Option<DateTime<Utc>>,
    pub block_number: Option<i64>,
    pub chain_status: Option<i64>,
    pub fee_paid: Option<String>,
    pub error: Option<String>,
}
