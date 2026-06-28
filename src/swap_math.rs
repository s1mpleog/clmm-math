use crate::{error::MathError, liquidity_math::LiquidityMath, sqrt_price_math::SqrtPriceMath};

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
    /// The amount of input that will be taken as fee
    pub fee_amount: u64,
}

impl SwapMath {
    // TODO: i have to add is_input: bool
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
        //
        // if a_to_b then price is decreasing √P_current > √P_target
        // else price is increase √P_current < √P_target

        let mut result = SwapStepResult::default();

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
                // update the next price
                // return the amount out, remaining amount and fee

                result.sqrt_price_next_x64 = sqrt_price_target;
                // amount_in = 2^64 - 1
                // if amount_in_to_reach_target was > 2^64 - 1, the the comparision
                // wound be false and code will fall to else branch
                // so its safe to cast this
                result.amount_in = amount_in_to_reach_target as u64;

                let amount_out = LiquidityMath::get_amount_1_for_liquidity(
                    sqrt_price_current,
                    sqrt_price_target,
                    liquidity,
                )?;

                result.amount_out = amount_out as u64;
                result.fee_amount = fee_amount;
            } else {
                // amount_in is small does not reaches target
                // the pool consumed user entier amount_in
                // we need to calculate new price based on amount in
                // we did not hit the extact √P_target so move as far as we can
                // return the info amount_in, amount_out, fee_amount, next_price

                let next_sqrt_price_x64 = SqrtPriceMath::get_next_sqrt_price_from_amount0(
                    sqrt_price_current,
                    liquidity,
                    amount_in,
                    true,
                )?;

                let amount_out = LiquidityMath::get_amount_1_for_liquidity(
                    sqrt_price_current,
                    next_sqrt_price_x64,
                    liquidity,
                )?;

                result.amount_in = amount_in;
                result.amount_out = amount_out as u64;
                result.fee_amount = fee_amount;
                result.sqrt_price_next_x64 = next_sqrt_price_x64;
            };
        }

        // TODO: work on y -> x

        unimplemented!()
    }

    fn calculate_amount_in_range(
        sqrt_price_current_x64: u128,
        sqrt_price_target_x64: u128,
        liquidity: u128,
        a_to_b: bool,
    ) -> Result<u128, MathError> {
        if a_to_b {
            LiquidityMath::get_amount_1_for_liquidity(
                sqrt_price_target_x64,
                sqrt_price_current_x64,
                liquidity,
            )
        } else {
            LiquidityMath::get_amount_0_for_liquidity(
                sqrt_price_current_x64,
                sqrt_price_target_x64,
                liquidity,
            )
        }
    }
}
