use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Serialize, sqlx::FromRow, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AllowanceDao {
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

#[derive(Serialize, sqlx::FromRow, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TokenTransferDao {
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

#[derive(Serialize, sqlx::FromRow, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChainTransferDao {
    pub id: i64,
    pub from_addr: String,
    pub receiver_addr: String,
    pub chain_id: i64,
    pub token_addr: Option<String>,
    pub token_amount: String,
    pub chain_tx_id: Option<i64>,
}

#[derive(Serialize, sqlx::FromRow, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TxDao {
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
    #[serde(skip_serializing)]
    pub call_data: Option<String>,
    pub created_date: DateTime<Utc>,
    pub first_processed: Option<DateTime<Utc>>,
    pub tx_hash: Option<String>,
    #[serde(skip_serializing)]
    pub signed_raw_data: Option<String>,
    pub signed_date: Option<DateTime<Utc>>,
    pub broadcast_date: Option<DateTime<Utc>>,
    pub broadcast_count: i64,
    pub confirm_date: Option<DateTime<Utc>>,
    pub block_number: Option<i64>,
    pub chain_status: Option<i64>,
    pub fee_paid: Option<String>,
    pub error: Option<String>,
    #[sqlx(default)]
    pub engine_message: Option<String>,
    #[sqlx(default)]
    pub engine_error: Option<String>,
}

#[derive(Serialize, sqlx::FromRow, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TxChainDao {
    pub id: i64,
    pub tx_hash: String,
    pub method: String,
    pub from_addr: String,
    pub to_addr: String,
    pub chain_id: i64,
    pub gas_limit: Option<i64>,
    pub max_fee_per_gas: Option<String>,
    pub priority_fee: Option<String>,
    pub val: String,
    pub nonce: i64,
    pub checked_date: DateTime<Utc>,
    pub blockchain_date: DateTime<Utc>,
    pub block_number: i64,
    pub chain_status: i64,
    pub fee_paid: String,
    pub error: Option<String>,
    #[sqlx(default)]
    pub engine_message: Option<String>,
    #[sqlx(default)]
    pub engine_error: Option<String>,
}
