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
    pub nonce: u64,
    pub signed_raw_data: Option<String>,
}
