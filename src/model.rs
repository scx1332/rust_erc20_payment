use chrono::{DateTime, Utc};

#[derive(sqlx::FromRow)]
#[derive(Debug, Clone)]
pub struct TokenTransfer {
    pub id: String,
    pub from_addr: String,
    pub receiver_addr: String,
    pub chain_id: i64,
    pub token_addr: Option<String>,
    pub token_amount: String,
    pub tx_id: Option<String>,
    pub fee_paid: Option<String>,
}

#[derive(sqlx::FromRow)]
#[derive(Debug, Clone)]
pub struct Web3TransactionDao {
    pub id: String,
    pub from_addr: String,
    pub to_addr: String,
    pub chain_id: i64,
    pub gas_limit: i64,
    pub max_fee_per_gas: String,
    pub priority_fee: String,
    pub val: String,
    pub nonce: Option<i64>,
    pub call_data: Option<String>,
    pub created_date: DateTime<Utc>,
    pub tx_hash: Option<String>,
    pub signed_raw_data: Option<String>,
    pub signed_date: Option<DateTime<Utc>>,
    pub broadcast_date: Option<DateTime<Utc>>,
    pub confirmed_date: Option<DateTime<Utc>>,
    pub block_number: Option<i64>,
    pub chain_status: Option<i64>,
    pub fee_paid: Option<String>,
}