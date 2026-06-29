use ruint::{Uint, aliases::U512};

use crate::{
    error::MathError,
    tick_math::{MAX_SQRT_PRICE_X64, MIN_SQRT_PRICE_X64},
};

pub struct SqrtPriceMath;

impl SqrtPriceMath {
    /// Given current sqrt price, liquidity and amount_1 find next sqrt price performs calculation
    /// in U512 space to avoid overflow in worst case scenario
    /// # Formula
    /// √P_new = (L * √P_current)/(L + (Δx * √P_current))
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

        /*
        * liquidity ∈ [1, 2^128 - 1]
        * liquidity_x64 * 2^64
        * = max(2^128 * 2^64)
        * = max(2^192 - 1)

        * sqrt_ratio_x64_256 ∈ [2^32, 2^96]
        * max = 2^192 * 2^96
        * = max(2^288)

        at worst case this will overflow multiplication so we have 2 options
        either fallback to slow formula √P_new = L/(L + (L / √P_current + Δx)) (slow because division
        is expesive on solana)
        so we have 2 path fast path -> √P_new = (L * √P_current)/(L + (Δx * √P_current)) overflows -> slow path

        or we can go with U512 which makes code small and cleaner at cost of some performance
        i choose to go with U512 option
        */

        let l_x64_512 = U512::from(liquidity) << 64;
        let sqrt_p_512 = U512::from(sqrt_ratio_x64);
        let amount_512 = U512::from(amount);

        // Q64.64 * Q64.64 = Q128.128
        let numerator: Uint<512, 8> = l_x64_512 * sqrt_p_512;

        // Q0.0 * Q64.64 = Q64.64
        let product = amount_512 * sqrt_p_512;

        // Q64.64 +- Q64.64 = Q64.64
        let denominator = if amount_specified_is_input {
            l_x64_512 + product
        } else {
            if l_x64_512 <= product {
                return Err(MathError::InsufficientLiquidity);
            }

            l_x64_512 - product
        };

        // Q128.128 / Q64.64 = Q64.64
        let mut price = numerator
            .checked_div(denominator)
            .ok_or(MathError::ZeroDenominator)?;

        if numerator % denominator != U512::ZERO {
            price += U512::from(1);
        }

        if price > MAX_SQRT_PRICE_X64 || price < MIN_SQRT_PRICE_X64 {
            return Err(MathError::SqrtPriceOutOfBounds);
        }

        // Q64.64
        Ok(u128::try_from(price).map_err(|_| MathError::Overflow)?)
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
