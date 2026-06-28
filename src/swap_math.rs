use crate::{error::MathError, liquidity_math::LiquidityMath};

pub struct SwapMath;

pub const FEE_DENOMINATOR: u32 = 1_000_000;

#[derive(Debug, Default)]
pub struct SwapStepResult {
    /// updated price
    pub sqrt_price_next_x64: u128,
    /// how much input token was consumed
    pub amount_in: u64,
    /// how much output token was sent to user
    pub amount_out: u64,
    pub fee_amount: u64,
}

impl SwapMath {
    pub fn compute_step(
        amount_remaining: u64,
        fee_rate: u32,
        liquidity: u128,
        sqrt_price_current: u128,
        sqrt_price_target: u128,
        a_to_b: bool,
    ) -> Result<(), MathError> {
        // what if liquidity is zero ?
        // what if sqrt_price_current > sqrt_price_target
        // what if amount_remaining is 0
        // what if fee rate is not valid ?

        if liquidity == 0 {
            return Err(MathError::ZeroLiquidity);
        }

        if sqrt_price_current == 0 || sqrt_price_target == 0 {
            return Err(MathError::ZeroSqrtPrice);
        }

        if amount_remaining == 0 {
            return Err(MathError::ZeroAmountSpecified);
        }

        // amount_remaining - Q0.0
        // liquidity - Q0.0
        // sqrt_price_current - Q64.64
        // sqrt_price_target - Q64.64

        // 1. compute how much token goes IN?
        // 2. compute how much token comes OUT?
        // 3. what is the exact new price?
        //
        // a_to_b = true then swapping x -> y
        // a_to_b = false then swapping y -> x

        if a_to_b {
            if sqrt_price_current <= sqrt_price_target {
                return Err(MathError::InvalidTickRange);
            }

            // amount_remaining ∈ [1, 2^64]
            // fee_rate ∈ [1, 2^32]
            // max = 2^32 * 2^64
            // max = 2^96
            // and 2^96 < 2^128 thus overflow is not possible
            let fee_amount =
                (amount_remaining as u128 * fee_rate as u128 / FEE_DENOMINATOR as u128) as u64;

            let amount_in = amount_remaining - fee_amount;

            let amount_in_to_reach_target = LiquidityMath::get_amount_0_for_liquidity(
                sqrt_price_target,
                sqrt_price_current,
                liquidity,
            )?;

            if (amount_in as u128) >= amount_in_to_reach_target {
                // amount_in reaches target with leftover
                todo!()
            } else {
                // amount_in is small does not reaches target
                todo!()
            };
        }

        unimplemented!()
    }
}
