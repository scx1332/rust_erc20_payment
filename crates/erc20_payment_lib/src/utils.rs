use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
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

fn compute_base(num_decimals: u32) -> rust_decimal::Decimal {
    if num_decimals == 18 {
        Decimal::new(1000000000000000000, 0)
    } else if num_decimals == 6 {
        Decimal::new(1000000, 0)
    } else {
        Decimal::from(10_u128.pow(num_decimals))
    }
}

///good from one gwei up to at least one billion ethers
pub fn rust_dec_to_u256(
    dec_amount: rust_decimal::Decimal,
    decimals: Option<u32>,
) -> Result<U256, ConversionError> {
    let num_decimals = decimals.unwrap_or(18);
    if num_decimals > 18 {
        return Err(ConversionError {
            msg: format!("Decimals: {num_decimals} cannot be greater than 18"),
        });
    }

    let dec_base = compute_base(num_decimals);
    //println!("dec: {}, number scale: {}", dec_base, dec_base.scale());

    let dec_mul = dec_amount.checked_mul(dec_base).ok_or(ConversionError {
        msg: "Overflow during conversion".to_string(),
    })?;
    //println!("number: {}, number scale: {}", dec_mul, dec_mul.scale());

    let dec_mul = dec_mul.normalize();
    //println!("number normalized: {}", dec_mul);

    if dec_mul.fract() != Decimal::from(0) {
        return Err(ConversionError::from(format!(
            "Number cannot have a fractional part {dec_mul}"
        )));
    }
    let u128 = dec_mul.to_u128().ok_or_else(|| {
        ConversionError::from(format!("Number cannot be converted to u128 {dec_mul}"))
    })?;
    Ok(U256::from(u128))
}

pub fn u256_to_rust_dec(
    amount: U256,
    decimals: Option<u32>,
) -> Result<rust_decimal::Decimal, ConversionError> {
    let num_decimals = decimals.unwrap_or(18);
    if num_decimals > 18 {
        return Err(ConversionError {
            msg: format!("Decimals: {num_decimals} cannot be greater than 18"),
        });
    }

    let dec_base = compute_base(num_decimals);

    Ok(Decimal::from(amount.as_u128()) / dec_base)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_rust_decimal_conversion() {
        let dec_gwei = Decimal::new(1, 18);
        let res = rust_dec_to_u256(dec_gwei, None).unwrap();
        assert_eq!(res, U256::from(1));

        let res = rust_dec_to_u256(dec_gwei / Decimal::from(2), None);
        println!("res: {res:?}");
        assert!(res.err().unwrap().msg.contains("fractional"));

        let res = rust_dec_to_u256(dec_gwei / Decimal::from(2), Some(19));
        println!("res: {res:?}");
        assert!(res.err().unwrap().msg.contains("greater than 18"));

        let res = rust_dec_to_u256(Decimal::from(8777666555_u64), None).unwrap();
        println!("res: {res:?}");
        assert_eq!(
            res,
            U256::from(8777666555_u64) * U256::from(1000000000000000000_u64)
        );

        let res = rust_dec_to_u256(Decimal::from(8777666555_u64) + dec_gwei, None).unwrap();
        println!("res: {res:?}");
        assert_eq!(res, U256::from(8777666555000000000000000001_u128));

        let res = rust_dec_to_u256(Decimal::from(0), None).unwrap();
        println!("res: {res:?}");
        assert_eq!(res, U256::from(0));

        let res = rust_dec_to_u256(Decimal::from(1), Some(0)).unwrap();
        println!("res: {res:?}");
        assert_eq!(res, U256::from(1));

        let res = rust_dec_to_u256(Decimal::from(1), Some(6)).unwrap();
        println!("res: {res:?}");
        assert_eq!(res, U256::from(1000000));

        let res = rust_dec_to_u256(Decimal::from(1), Some(9)).unwrap();
        println!("res: {res:?}");
        assert_eq!(res, U256::from(1000000000));

        let res =
            rust_dec_to_u256(Decimal::from_str("123456789.123456789").unwrap(), Some(18)).unwrap();
        println!("res: {res:?}");
        assert_eq!(
            res,
            U256::from_dec_str("123456789123456789000000000").unwrap()
        );

        //this should result in overflow, because 79228162514264337593543950336 == 2**96
        let res = rust_dec_to_u256(
            Decimal::from_str("79228162514.264337593543950336").unwrap(),
            Some(18),
        );
        println!("res: {res:?}");
        assert!(res.err().unwrap().msg.to_lowercase().contains("overflow"));

        //this is the max value that can be represented by rust decimal
        let res = rust_dec_to_u256(
            Decimal::from_str("79228162514.264337593543950335").unwrap(),
            Some(18),
        )
        .unwrap();
        println!("res: {res:?}");
        assert_eq!(res, U256::from(79228162514264337593543950335_u128));

        //this is the max value that can be represented by rust decimal
        let res = rust_dec_to_u256(
            Decimal::from_str("79228162514264337593543950335").unwrap(),
            Some(0),
        )
        .unwrap();
        println!("res: {res:?}");
        assert_eq!(res, U256::from(79228162514264337593543950335_u128));

        //this is the max value that can be represented by rust decimal
        let res = rust_dec_to_u256(
            Decimal::from_str("79228162514264337593543.950335").unwrap(),
            Some(6),
        )
        .unwrap();
        println!("res: {res:?}");
        assert_eq!(res, U256::from(79228162514264337593543950335_u128));

        //this is the max value that can be represented by rust decimal
        let res = rust_dec_to_u256(
            Decimal::from_str("792281625142643.37593543950335").unwrap(),
            Some(14),
        )
        .unwrap();
        println!("res: {res:?}");
        assert_eq!(res, U256::from(79228162514264337593543950335_u128));
        //assert_eq!(res, U256::zero());
    }
}
