use std::error::Error;
use std::fmt::{Display, Formatter};
use web3::types::U256;

#[derive(Debug, Clone)]
pub struct ConversionError {
    pub msg: String,
}

impl ConversionError {
    pub fn from(msg: String) -> Self {
        Self { msg }
    }
}

impl Display for ConversionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error during conversion: {}", self.msg)
    }
}

impl Error for ConversionError {
    fn description(&self) -> &str {
        "Conversion error"
    }
}

pub fn gwei_to_u256(gas: f64) -> Result<U256, ConversionError> {
    pub const GWEI: f64 = 1.0E9;
    if gas < 0.0 {
        //return Err(ConversionError"Gas price cannot be negative");
        return Err(ConversionError {
            msg: "Gas price cannot be negative".to_string(),
        });
    }
    if gas > 1.0E9 {
        return Err(ConversionError {
            msg: "Gas price cannot be greater than 1E9".to_string(),
        });
    }
    if gas.is_nan() {
        return Err(ConversionError {
            msg: "Gas price cannot be NaN".to_string(),
        });
    }
    Ok(U256::from((gas * GWEI) as u64))
}
