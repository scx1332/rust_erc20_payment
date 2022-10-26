use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct Web3TransactionDao {
    pub unique_id: String,
    pub from_addr: String,
    pub to_addr: String,
    pub chain_id: u64,
    pub gas_limit: u64,
    pub max_fee_per_gas: String,
    pub priority_fee: String,
    pub val: String,
    pub nonce: Option<u64>,
    pub call_data: Option<String>,
    pub created_date: DateTime<Utc>,
    pub tx_hash: Option<String>,
    pub signed_raw_data: Option<String>,
    pub signed_date: Option<DateTime<Utc>>,
    pub broadcast_date: Option<DateTime<Utc>>,
    pub confirmed_date: Option<DateTime<Utc>>,
    pub block_number: Option<u64>,
    pub chain_status: Option<u64>,
    pub fee_paid: Option<String>,
}
