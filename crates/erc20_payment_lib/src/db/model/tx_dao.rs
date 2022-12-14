use chrono::{DateTime, Utc};
use serde::Serialize;

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
