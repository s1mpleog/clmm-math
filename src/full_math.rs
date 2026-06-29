use ethnum::U256;

use crate::error::MathError;

/// Multiplies two u128 (Q64.64) numbers in U256 space to avoid overflow
/// casts it back from Q128.128 to Q64.64 and returns as u128
/// # Arguments
/// `a`: u128
/// `b`: u128
#[inline(always)]
pub fn mul_shift_64(a: u128, b: u128) -> u128 {
    let result = U256::from(a) * U256::from(b);
    // cast it back to Q64.64 from Q128.128
    (result >> 64u32).as_u128()
}

/// performs calculation in u256 space
/// since multiplying two u128 numbers overflow to u256, 2^128 * 2^128 = 2^256
/// # Arguments
/// `a` - LHS in Q64.64
/// `b` - RHS in Q64.64
/// `denom` - denominator
///
/// Throws if denom = 0
#[inline(always)]
pub fn mul_div_floor(a: u128, b: u128, denom: u128) -> Result<u128, MathError> {
    let result = U256::from(a)
        .checked_mul(U256::from(b))
        .ok_or(MathError::Overflow)?
        .checked_div(U256::from(denom))
        .ok_or(MathError::ZeroDenominator)?;

    Ok(result.as_u128())
}

#[inline(always)]
pub fn mul_div_ceil(a: u128, b: u128, denom: u128) -> Option<u128> {
    if denom == 0 {
        return None;
    }

    let val = a.checked_mul(b)?;
    let mut result = val / denom;

    if val % denom != 0 {
        result += 1;
    }

    Some(result)
}
