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
    ) -> Result<SwapStepResult, MathError> {
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

        if fee_rate >= FEE_DENOMINATOR {
            return Err(MathError::InvalidFeeRate);
        }

        // amount_remaining - Q0.0
        // fee_rate - Q0.0
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

        // amount_remaining ∈ [1, 2^64]
        // FEE_DENOMINATOR ∈ [2^20]
        // fee_rate ∈ [0, 2^20] (because of invariant fee_rate < FEE_DENOMINATOR)
        //
        // FEE_DENOMINATOR - fee_rate = (2^20 - 2^20) = max 2^20 (temp)
        // amount_remaining * temp = (2^64 * 2^20) = max 2^84 (temp_1)
        // temp_1 / FEE_DENOMINATOR = (2^84 / 2^20) = max 2^64
        // this proving even in worse case scenario it will never overflow u64
        // so it's completely safe to cast to u64.
        let amount_remaining_less_fee = ((amount_remaining as u128
            * (FEE_DENOMINATOR as u128 - fee_rate as u128))
            / FEE_DENOMINATOR as u128) as u64;

        if a_to_b {
            // user is giving token X and they want token Y
            // since the token X is increasing the price must decrease

            if sqrt_price_current <= sqrt_price_target {
                return Err(MathError::InvalidTickRange);
            }

            // how much tokenX is need to reach the target price
            let amount_in_to_reach_target = LiquidityMath::get_amount_0_for_liquidity(
                sqrt_price_target,
                sqrt_price_current,
                liquidity,
            )?;

            // is amount_in more than enough to reach target ?
            if (amount_remaining_less_fee as u128) >= amount_in_to_reach_target {
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
                // TODO: understand how this works
                result.fee_amount = Self::calculate_fee_amount(result.amount_in, fee_rate)?;
            } else {
                // amount_in is small does not reaches target
                // the pool consumed user entier amount_in
                // we need to calculate new price based on amount in
                // we did not hit the extact √P_target so move as far as we can
                // return the info amount_in, amount_out, fee_amount, next_price

                let next_sqrt_price_x64 = SqrtPriceMath::get_next_sqrt_price_from_amount0(
                    sqrt_price_current,
                    liquidity,
                    amount_remaining_less_fee,
                    true,
                )?;

                let amount_out = LiquidityMath::get_amount_1_for_liquidity(
                    sqrt_price_current,
                    next_sqrt_price_x64,
                    liquidity,
                )?;

                result.amount_in = amount_remaining_less_fee;
                result.amount_out = amount_out as u64;
                result.fee_amount = amount_remaining - result.amount_in;
                result.sqrt_price_next_x64 = next_sqrt_price_x64;
            };
        } else {
            // user gives token y then want token x
            // token x is decreasing so the price will increase
            // the price must increase
            // input token: token 1 (Y)
            // output token: token 0 (X)
            //
            // does user have enough token to reach target ?
            // or user does not have enough token to reach target ?

            // If swapping Y for X, price goes UP.
            // Current price must be strictly less than target price.
            if sqrt_price_current >= sqrt_price_target {
                return Err(MathError::InvalidTickRange);
            }

            // how much token Y is required to move from current to target
            let amount_in_to_reach_target = LiquidityMath::get_amount_1_for_liquidity(
                sqrt_price_target,
                sqrt_price_current,
                liquidity,
            )?;

            if (amount_remaining_less_fee as u128) >= amount_in_to_reach_target {
                // they have more than enough token to move price to target

                // how much token0 do we need to move the price
                let amount_out = LiquidityMath::get_amount_0_for_liquidity(
                    sqrt_price_current,
                    sqrt_price_target,
                    liquidity,
                )?;

                result.sqrt_price_next_x64 = sqrt_price_target;
                result.amount_in = amount_in_to_reach_target as u64;
                result.fee_amount = Self::calculate_fee_amount(result.amount_in, fee_rate)?;
                result.amount_out = amount_out as u64;
            } else {
                // they don't have enough token to reach target

                // find how far can they go
                let next_sqrt_price = SqrtPriceMath::get_next_sqrt_price_from_amount1(
                    sqrt_price_current,
                    liquidity,
                    amount_remaining_less_fee,
                )?;

                // how much token0 we need to go from current to target
                let amount_out = LiquidityMath::get_amount_0_for_liquidity(
                    sqrt_price_current,
                    next_sqrt_price,
                    liquidity,
                )?;

                result.amount_in = amount_remaining_less_fee;
                result.amount_out = amount_out as u64;
                result.fee_amount = amount_remaining - result.amount_in;
                result.sqrt_price_next_x64 = next_sqrt_price;
            }
        }

        Ok(result)
    }

    fn calculate_fee_amount(amount_used: u64, fee_rate: u32) -> Result<u64, MathError> {
        let denom = FEE_DENOMINATOR as u128;
        let rate = fee_rate as u128;

        // If the fee rate is >= the denominator, the math is invalid.
        if rate >= denom {
            return Err(MathError::InvalidFeeRate);
        }

        let used = amount_used as u128;
        let divisor = denom - rate;

        let numerator = used.checked_mul(rate).ok_or(MathError::Overflow)?;

        let fee = (numerator + divisor - 1) / divisor;

        Ok(fee as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tick_math::TickMath;

    #[test]
    fn test_swap_step_matrix() {
        let sqrt_price_current = TickMath::tick_to_sqrt_price_x64(0).unwrap();
        let liquidity = 1_000_000_000_000u128;
        let fee_rate = 300u32; // 0.3%
        let fee_denom = 1_000_000u32;

        // Test both directions
        let directions = [true, false];

        for a_to_b in directions {
            // Set a target price 100 ticks away
            let sqrt_price_target = if a_to_b {
                TickMath::tick_to_sqrt_price_x64(-100).unwrap()
            } else {
                TickMath::tick_to_sqrt_price_x64(100).unwrap()
            };

            // --- Scenario 1: Huge Bag (Reaches Target) ---
            let huge_amount = 1_000_000_000_000u64;
            let result = SwapMath::compute_step(
                huge_amount,
                fee_rate,
                liquidity,
                sqrt_price_current,
                sqrt_price_target,
                a_to_b,
            )
            .unwrap();

            // 1. Price must hit exact target
            assert_eq!(
                result.sqrt_price_next_x64, sqrt_price_target,
                "Price should hit target"
            );

            // 2. We should NOT consume the entire bag
            assert!(
                result.amount_in + result.fee_amount < huge_amount,
                "Should have leftovers"
            );

            // 3. Fee should be proportionally correct (rounding up)

            let expected_fee = ((result.amount_in as u128 * fee_rate as u128
                + (fee_denom as u128 - fee_rate as u128 - 1))
                / (fee_denom as u128 - fee_rate as u128)) as u64;

            assert_eq!(
                result.fee_amount, expected_fee,
                "Fee should match amount_in used"
            );

            // --- Scenario 2: Small Bag (Stops Partway) ---
            let small_amount = 1_000u64;
            let result = SwapMath::compute_step(
                small_amount,
                fee_rate,
                liquidity,
                sqrt_price_current,
                sqrt_price_target,
                a_to_b,
            )
            .unwrap();

            // 1. Price must NOT hit target
            if a_to_b {
                assert!(
                    result.sqrt_price_next_x64 > sqrt_price_target,
                    "Price should stop before target (down)"
                );
            } else {
                assert!(
                    result.sqrt_price_next_x64 < sqrt_price_target,
                    "Price should stop before target (up)"
                );
            }

            // 2. We MUST consume the entire bag (amount_in + fee = amount_remaining)
            assert_eq!(
                result.amount_in + result.fee_amount,
                small_amount,
                "Should consume entire small bag"
            );

            // 3. Verify the math by cross-checking the output amount with LiquidityMath
            let expected_out = if a_to_b {
                LiquidityMath::get_amount_1_for_liquidity(
                    sqrt_price_current,
                    result.sqrt_price_next_x64,
                    liquidity,
                )
                .unwrap()
            } else {
                LiquidityMath::get_amount_0_for_liquidity(
                    sqrt_price_current,
                    result.sqrt_price_next_x64,
                    liquidity,
                )
                .unwrap()
            };
            assert_eq!(
                result.amount_out as u128, expected_out,
                "Output math must match price movement"
            );
        }
    }
}
