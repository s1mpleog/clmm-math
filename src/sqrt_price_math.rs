use ethnum::U256;

use crate::{
    error::MathError,
    tick_math::{MAX_SQRT_PRICE_X64, MIN_SQRT_PRICE_X64},
};

pub struct SqrtPriceMath;

impl SqrtPriceMath {
    /// Given current sqrt price, liquidity and amount_1 find next sqrt price
    /// # Formula
    /// if sqrt price * liquidity do not overflow u256 then
    /// √P_new = (L * √P_current)/(L + (Δx * √P_current))
    /// else √P_new = L/(L + (L / √P_current + Δx))
    ///
    /// # Arguments
    /// `sqrt_ratio_x64` - current sqrt price (Q64.64)
    /// `liquidity` - (Q0.0)
    /// `amount_0` - (Q0.0)
    ///
    /// Throws if sqrt_ratio_x64 > MAX_SQRT_PRICE_X64
    ///
    /// if liquidity == 0 or sqrt_ratio_x64 then it returns sqrt_ratio_x64
    ///
    /// # Return next_sqrt_price in Q64.64 format

    // TODO: for some reason i am not fully stasified with this implementation
    // either use u512 which will significantly reduces complexity
    // or write helper functions to reduce the code complexity
    pub fn get_next_sqrt_price_from_amount0(
        sqrt_ratio_x64: u128,
        liquidity: u128,
        amount: u64,
        amount_specified_is_input: bool,
    ) -> Result<u128, MathError> {
        if amount == 0 || liquidity == 0 {
            return Ok(sqrt_ratio_x64);
        }

        if sqrt_ratio_x64 > MAX_SQRT_PRICE_X64 || sqrt_ratio_x64 < MIN_SQRT_PRICE_X64 {
            return Err(MathError::SqrtPriceOutOfBounds);
        }

        // Q64.64
        let liquidity_x64 = U256::from(liquidity) << 64_u32;
        // Q64.64
        let sqrt_ratio_x64_256 = U256::from(sqrt_ratio_x64);

        // Q64.64
        let price =
              // Q128.128
              // liquidity ∈ [1, 2^128 - 1]
              // liquidity_x64 * 2^64
              // = max(2^128 * 2^64)
              // = max(2^192 - 1)
              //
              // sqrt_ratio_x64_256 ∈ [2^32, 2^96]
              // max = 2^192 * 2^96
              // = max(2^288)
              //
              // the worse case requires 2^288 which can not fit in 2^55 the multiplication
              // will overflow
            if let Some(numerator) = liquidity_x64.checked_mul(sqrt_ratio_x64_256) {
                // Q64.64
                // amount_0 - Q0.0
                // sqrt_ratio_x64 - Q64.64
                // amount_0 * sqrt_ratio_x64 = 2^0 * 2^64 = 2^64 (Q64.64)
                // Q64.64 / Q0.0 = 2^64/2^0 = 2^64/1 = 2^64 (Q64.64)

                // amount ∈ [1, 2^64 + 1]
                // sqrt_ratio_x64 ∈ [2^32, 2^96]
                // max = (2^64 + 1) * 2^96
                // = 2^160 + 1
                // max(2^160)
                // and since 2^160 < 2^256 thus proving this will never overflow 2^256
                let product = U256::from(amount) * sqrt_ratio_x64;

                // in both case denominator - Q64.64
                let denominator = if amount_specified_is_input {
                    liquidity_x64 + product
                } else {
                    // (L * sqrtP) / (L - (amount * sqrtP))

                    if liquidity_x64 < product {
                        return Err(MathError::ZeroDenominator);
                    }

                    // liquidity ∈ [1, 2^192]
                    // product   ∈ [1, 2^160]
                    //
                    // 2^192 - 2^160
                    // = max(2^192)
                    //
                    // this proving liquidity_x64 will not overflow or underflow because of this
                    // subtraction

                    liquidity_x64 - product
                };
                // Q128.128 / Q64.64 = Q64.64
                // 2^128 / 2^64 = 2^64
                let mut price = numerator
                    .checked_div(denominator)
                    .ok_or(MathError::ZeroDenominator)?;

                // ROUND UP to protect the pool!
                if numerator % denominator != U256::ZERO {
                    price += 1;
                }
                price
            } else {
                // * If Δx * √P overflows, use formula `√P' = L / (L/√P + Δx)`

                // liquidity_x64 - Q64.64
                // sqrt_ratio_x64_256 - Q64.64
                // amount - Q0.0
                //
                // Q64.64 / Q64.64 = Q0.0
                // Q0.0 + Q0.0 = Q0.0
                // Q0.0 - Q0.0 = Q0.0
                let denominator = if amount_specified_is_input {
                    liquidity_x64
                    .checked_div(sqrt_ratio_x64_256)
                    .ok_or(MathError::ZeroDenominator)?
                    .checked_add(U256::from(amount))
                    .ok_or(MathError::Overflow)?
                } else {
                    liquidity_x64
                           .checked_div(sqrt_ratio_x64_256)
                           .ok_or(MathError::ZeroDenominator)?
                           .checked_sub(U256::from(amount))
                           .ok_or(MathError::Overflow)?
                };

                // Q64.64 / Q0.0 = Q64.64
                let mut price = liquidity_x64
                    .checked_div(denominator)
                    .ok_or(MathError::ZeroDenominator)?;

                // ROUND UP to protect the pool!
                if liquidity_x64 % denominator != U256::ZERO {
                    price += 1;
                }
                price
            };

        if price > MAX_SQRT_PRICE_X64 || price < MIN_SQRT_PRICE_X64 {
            return Err(MathError::SqrtPriceOutOfBounds);
        }

        // Q64.64
        Ok(price.as_u128())
    }

    /// Given current sqrt price, liquidity and amount_1 find next sqrt price
    /// # Formula
    /// √P_new = √P_current + Δy/L
    ///
    /// # Arguments
    /// `sqrt_ratio_x64` - current sqrt price (Q64.64)
    /// `liquidity` - (Q0.0)
    /// `amount_1` - (Q0.0)
    ///
    /// Throws if sqrt_ratio_x64 > MAX_SQRT_PRICE_X64
    ///
    /// if liquidity == 0 or sqrt_ratio_x64 then it returns sqrt_ratio_x64
    ///
    /// # Return next_sqrt_price in Q64.64 format
    pub fn get_next_sqrt_price_from_amount1(
        sqrt_ratio_x64: u128,
        liquidity: u128,
        amount_1: u64,
    ) -> Result<u128, MathError> {
        if amount_1 == 0 || liquidity == 0 {
            return Ok(sqrt_ratio_x64);
        }

        if sqrt_ratio_x64 > MAX_SQRT_PRICE_X64 || sqrt_ratio_x64 < MIN_SQRT_PRICE_X64 {
            return Err(MathError::SqrtPriceOutOfBounds);
        }

        // sqrt_ratio_x64 - Q64.64
        // liquidity: Q0.0
        // amount_1: Q0.0

        // Q64.64/Q0.0 = Q64.64
        //
        // amount_1 ∈ [1, 2^64]
        // doing << 64 is 2^64
        // so max = 2^64 * 2^64 = 2^128
        // max = 2^128
        // so even at worst case doing (amount_1 as u128) << 64 will never overflow
        let price_change = ((amount_1 as u128) << 64)
            .checked_div(liquidity)
            .ok_or(MathError::ZeroDenominator)?;

        // Q64.64 + Q64.64 = Q64.64
        // new_price = Q64.64
        Ok(sqrt_ratio_x64
            .checked_add(price_change)
            .ok_or(MathError::Overflow)?)
    }
}

#[cfg(test)]
mod sqrt_test {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn fuzz_get_next_sqrt_price_from_amount0(
            sqrt_p in MIN_SQRT_PRICE_X64..MAX_SQRT_PRICE_X64,
            liquidity in 1u128..u128::MAX,
            amount in 1u64..u64::MAX,
            is_input in proptest::bool::ANY,
        ) {
            let result = SqrtPriceMath::get_next_sqrt_price_from_amount0(
                sqrt_p,
                liquidity,
                amount,
                is_input,
            );

            // If it returns an error (like ZeroDenominator on massive withdrawals),
            // that's fine! We just don't want a panic/overflow.
            if let Ok(new_price) = result {

                // 2. Check Directional Logic
                if is_input {
                    // Adding Token 0 -> Price goes DOWN
                    prop_assert!(new_price <= sqrt_p, "Price should decrease on input");
                } else {
                    // Removing Token 0 -> Price goes UP
                    prop_assert!(new_price >= sqrt_p, "Price should increase on output");
                }

                // 3. Check Bounds
                prop_assert!(new_price >= MIN_SQRT_PRICE_X64 && new_price <= MAX_SQRT_PRICE_X64);
            }
        }
    }
}
