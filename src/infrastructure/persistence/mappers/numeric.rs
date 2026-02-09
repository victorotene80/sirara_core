use bigdecimal::{BigDecimal, ToPrimitive};
use crate::domain::repository::RepoError;

pub fn bd_to_i128(bd: &BigDecimal, field: &'static str) -> Result<i128, RepoError> {
    if let Some(v) = bd.to_i128() {
        return Ok(v);
    }

    let s = bd.to_string();
    Err(RepoError::Integrity {
        message: format!("{field} must be integer (scale=0) and within i128 range, got: {s}"),
    })
}


pub fn i128_to_bd(v: i128) -> BigDecimal {
    BigDecimal::from(v)
}

pub fn numeric0_to_i128_strict(v: &BigDecimal, account_id: i64) -> Result<i128, RepoError> {
    let s = v.to_string();
    if s.contains('.') {
        return Err(RepoError::Integrity {
            message: format!("non-integer numeric found (account_id={account_id})"),
        });
    }
    s.parse::<i128>().map_err(|_| RepoError::Integrity {
        message: format!("numeric out of i128 range (account_id={account_id})"),
    })
}