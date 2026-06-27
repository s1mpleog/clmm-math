use ethnum::U256;

use crate::{
    error::MathError,
    tick_math::{MAX_SQRT_PRICE_X64, MIN_SQRT_PRICE_X64},
};

pub struct SqrtPriceMath;

impl SqrtPriceMath {
    /// Given current sqrt price, liquidity and amount_1 find next sqrt price
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

        // sqrt_ratio_x64 - Q64.64
        // liquidity - Q64.64
        // amount_0 - Q64.64

        // formula is  √P_new = (L * √P_current)/(L + (Δx * √P_current))
        // sqrt_ratio_x64 - Q64.64 lets call its crr
        // liquidity - Q64.64 this l
        // amount_0 - Q64.64  and this a

        // now when we do l * curr this will give us Q0.0 * Q64.64 = Q64.64 (numerator)
        // then we will do a * curr = Q0.0 * Q64.64 = Q64.64 (temp)
        // and then l_x64 + temp = Q64.64 (denominator)
        // numerator / denominator = Q64.64 / Q64.64 = Q0.0
        // how is it possible to preserve Q64.64 ?
        //
        // actually what if we do
        // l_x64 * curr = Q64.64 * Q64.64 = Q128.128 (numerator)
        // then we will do a * curr = Q0.0 * Q64.64 = Q64.64 (temp)
        // and then l_x64 + temp = Q64.64 (denominator)
        // and then numerator / denominator = Q128.128 / Q64.64 = Q64.64
        // but doing liquidity << 64 is not generally safe it can overflow 2^128
        // so for safety let's cast it to u256

        // Q64.64
        let liquidity_x64 = U256::from(liquidity) << 64_u32;
        // Q64.64
        let sqrt_ratio_x64_256 = U256::from(sqrt_ratio_x64);

        // FIXME: this will overflow
        // our MAX_SQRT_PRICE takes 2^96 bit
        // and liquidity_x64 can be 2^192 here is why
        // max_liquidity = 2^128 - 1 << 64
        // = 2^192
        // 192 + 96 = 288 bits which will overflow numerator 256 bits
        // to prevent this use slow formula √P_new = L/(L/√P + amount)

        // Q128.128
        // liquidity_x64 - Q64.64
        // sqrt_ratio_x64_256 - Q64.64
        // Q64.64 * Q64.64 = (2^64 * 2^64) = 2^128 (Q128.128)
        let numerator = liquidity_x64
            .checked_mul(sqrt_ratio_x64_256)
            .ok_or(MathError::Overflow)?;

        // Q64.64
        // amount_0 - Q0.0
        // sqrt_ratio_x64 - Q64.64
        // amount_0 * sqrt_ratio_x64 = 2^0 * 2^64 = 2^64 (Q64.64)
        // Q64.64 / Q0.0 = 2^64/2^0 = 2^64/1 = 2^64 (Q64.64)
        let product = U256::from(amount)
            .checked_mul(U256::from(sqrt_ratio_x64))
            .ok_or(MathError::Overflow)?;

        // Q64.64
        // Q64.64 + Q64.64 = Q64.64
        // Q64.64 - Q64.64 = Q64.64
        let denominator = if amount_specified_is_input {
            liquidity_x64
                .checked_add(product)
                .ok_or(MathError::Overflow)?
        } else {
            if liquidity_x64 <= product {
                return Err(MathError::ZeroDenominator);
            }
            liquidity_x64
                .checked_sub(product)
                .ok_or(MathError::Overflow)?
        };

        // Q128.128 / Q64.64 = Q64.64
        // 2^128 / 2^64 = 2^64
        let price = numerator
            .checked_div(denominator)
            .ok_or(MathError::ZeroDenominator)?;

        // 6. Bounds check
        if price < MIN_SQRT_PRICE_X64 || price > MAX_SQRT_PRICE_X64 {
            return Err(MathError::SqrtPriceOutOfBounds);
        }

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
