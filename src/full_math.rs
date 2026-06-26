use ethnum::U256;

#[inline(always)]
/// Multiplies two u128 numbers in U256 space to avoid overflow
/// casts it back to Q128.128 to Q64.64 and returns as u128
/// # Arguments
/// `a`: u128
/// `b`: u128
pub fn mul_shift_64(a: u128, b: u128) -> u128 {
    let result = U256::from(a) * U256::from(b);
    // cast it back to Q64.64 from Q128.128
    (result >> 64u32).as_u128()
}
