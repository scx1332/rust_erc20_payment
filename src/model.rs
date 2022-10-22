use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct Web3TransactionDao {
    pub from: String,
    pub to: String,
    pub chain_id: u64,
    pub gas_limit: u64,
    pub total_fee: String,
    pub priority_fee: String,
    pub value: String,
    pub data: Option<String>,
    pub nonce: Option<u64>,
    pub tx_hash: Option<String>,
    pub signed_raw_data: Option<String>,
    pub created_date: DateTime<Utc>,
    pub signed_date: Option<DateTime<Utc>>,
    pub broadcast_date: Option<DateTime<Utc>>,
    pub confirmed_date: Option<DateTime<Utc>>,
    pub block_number: Option<u64>,
    pub chain_status: Option<u64>,
    pub fee_paid: Option<String>,
}
