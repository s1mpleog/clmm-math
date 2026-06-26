use crate::{error::MathError, full_math::mul_div_floor};

pub struct LiquidityMath;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TokenAmounts {
    pub amount_0: u128,
    pub amount_1: u128,
}

impl LiquidityMath {
    /// Given tick_lower, tick_upper and amount 0 calculate liquidity L
    /// # Formula
    /// L = x * √(P_upper * √P_lower) / √(P_upper - √P_lower)
    ///
    /// # Arguments
    /// `sqrt_ratio_a_x64` lp lower bound in sqrt space Q64.64
    /// `sqrt_ratio_b_x64` lp upper bound in sqrt space Q64.64
    /// `amount_0` amount of token 0
    ///
    /// # Note
    /// returns liquidity in Q0.0 fixed point scaling not Q64.64 so handle accordingly
    pub fn get_liquidity_from_amount_0(
        mut sqrt_ratio_a_x64: u128,
        mut sqrt_ratio_b_x64: u128,
        amount_0: u64,
    ) -> Result<u128, MathError> {
        if amount_0 == 0 {
            return Ok(0);
        }

        // lower bound must not be greater than upper bound
        if sqrt_ratio_a_x64 > sqrt_ratio_b_x64 {
            std::mem::swap(&mut sqrt_ratio_a_x64, &mut sqrt_ratio_b_x64);
        }

        // calculates √Pa * √Pb into U256 space and scales it back to u128 but does not change
        // fixed point representation before multiplication sqrt_ratio and and b had in Q64.64
        // after multiplication it's in Q128.128 so we divide with 1u128 << 64 to scale back to Q64.64
        //
        // multiplying two Q64.64 number scales it to Q128.128
        // dividing Q128.128 with Q64.64 scales it back to Q64.64
        // dividing Q64.64 number with Q64.64 scales it to Q0.0
        let intermediate = mul_div_floor(sqrt_ratio_a_x64, sqrt_ratio_b_x64, 1u128 << 64)?;

        let delta = sqrt_ratio_b_x64
            .checked_sub(sqrt_ratio_a_x64)
            .ok_or(MathError::Overflow)?;

        // intermediate is in Q64.64
        // amount_0 is in Q0.0
        // delta is in Q64.64
        // amount_0 * intermediate = Q0.0 * Q64.64 = Q64.64
        // Q64.64 / Q64.64 = Q0.0
        Ok(mul_div_floor(u128::from(amount_0), intermediate, delta))?
    }

    /// Given Lp lower and upper price range and amount_1
    /// calculate liquidity
    /// # Formula
    /// L = y / (√P_upper - √P_lower)
    ///
    /// # Arguments
    /// `sqrt_ratio_a_x64` - lower price range in Q64.64 scaling
    /// `sqrt_ratio_b_x64` - upper price range in Q64.64 scaling
    /// `amount_1` - token y in Q0.0 scaling
    pub fn get_liquidity_from_amount_1(
        mut sqrt_ratio_a_x64: u128,
        mut sqrt_ratio_b_x64: u128,
        amount_1: u64,
    ) -> Result<u128, MathError> {
        if amount_1 == 0 {
            return Ok(0);
        }

        // sqrt_ratio_a_x64 should be smaller
        if sqrt_ratio_a_x64 > sqrt_ratio_b_x64 {
            std::mem::swap(&mut sqrt_ratio_a_x64, &mut sqrt_ratio_b_x64);
        }

        let delta = sqrt_ratio_b_x64
            .checked_sub(sqrt_ratio_a_x64)
            .ok_or(MathError::Overflow)?;

        // amount_1: Q0.0
        // 1u128 << 64: Q64.64
        // delta: Q64.64
        // multiplying amount_1 * (1u128 << 64) scales it to Q64.64
        // then dividing with delta Q64.64 scales it back to Q0.0
        Ok(mul_div_floor(u128::from(amount_1), 1u128 << 64, delta))?
    }

    /// Calculates the maximum amount of liquidity for given amount_0, amount_1 the current pool price
    /// and the price range
    ///
    /// # Formula
    /// 1. if pool_price <= lower price range then return token_0 only by calling `get_liquidity_from_amount_0`
    /// 2. if pool_price < price upper range then return token_1 only by calling `get_liquidity_from_amount_1`
    /// 3. return the minimum token_0 and token_1
    ///
    /// # Arguments
    /// `sqrt_ratio_x64` - current pool price in Q64.64
    /// `sqrt_ratio_a_x64` - lower price range in Q64.64
    /// `sqrt_ratio_b_x64` - upper price range in Q64.64
    /// `amount_0` - token 0 amount in Q0.0
    /// `amount_1` - token 1 amount in Q0.0
    pub fn get_liquidity_from_amounts(
        sqrt_ratio_x64: u128,
        mut sqrt_ratio_a_x64: u128,
        mut sqrt_ratio_b_x64: u128,
        amount_0: u64,
        amount_1: u64,
    ) -> Result<u128, MathError> {
        if sqrt_ratio_a_x64 > sqrt_ratio_b_x64 {
            std::mem::swap(&mut sqrt_ratio_a_x64, &mut sqrt_ratio_b_x64);
        }

        if sqrt_ratio_x64 <= sqrt_ratio_a_x64 {
            // only token 0 is active
            // Pc <= Pa
            Self::get_liquidity_from_amount_0(sqrt_ratio_a_x64, sqrt_ratio_b_x64, amount_0)
        } else if sqrt_ratio_x64 < sqrt_ratio_b_x64 {
            // now at this stage we know first branch is false so Pc > Pa
            // and since we are here we know second branch is true so Pc < Pb
            // Pc < Pa < Pb

            Ok(u128::min(
                Self::get_liquidity_from_amount_0(sqrt_ratio_x64, sqrt_ratio_b_x64, amount_0)?,
                Self::get_liquidity_from_amount_1(sqrt_ratio_a_x64, sqrt_ratio_x64, amount_1)?,
            ))
        } else {
            // Pc > Pa and Pc > Pb
            // so only token 1 is active
            Self::get_liquidity_from_amount_1(sqrt_ratio_a_x64, sqrt_ratio_b_x64, amount_1)
        }
    }

    /// Calculate how many token_0 user will get given liquidity, lower and upper bound
    /// # Formula
    /// x = L * (√P_upper - √P_lower) / (√P_upper * √P_lower)
    ///
    /// # Arguments
    /// `sqrt_ratio_a_x64` - lower tick bound in Q64.64
    /// `sqrt_ratio_b_x64` - upper tick bound in Q64.64
    /// `liquidity` - liquidity in given tick range in Q0.0
    ///
    /// Throws if liquidity = 0
    ///
    /// # Return
    /// token_1 in Q0.0
    pub fn get_amount_0_for_liquidity(
        mut sqrt_ratio_a_x64: u128,
        mut sqrt_ratio_b_x64: u128,
        liquidity: u128,
    ) -> Result<u128, MathError> {
        if liquidity == 0 {
            return Err(MathError::ZeroLiquidity);
        }

        if sqrt_ratio_a_x64 == sqrt_ratio_b_x64 {
            return Ok(0); // A zero-width range requires 0 tokens!
        }

        if sqrt_ratio_a_x64 > sqrt_ratio_b_x64 {
            std::mem::swap(&mut sqrt_ratio_a_x64, &mut sqrt_ratio_b_x64);
        }

        // Q64.64 - Q64.64 = Q64.64
        let delta = sqrt_ratio_b_x64
            .checked_sub(sqrt_ratio_a_x64)
            .ok_or(MathError::Overflow)?;

        // scale it to Q64.64
        let liquidity_x64 = liquidity << 64;

        // Q64.64 * Q64.64 = Q128.128
        // Q128.128 / Q64.64 = Q64.64
        let numerator = mul_div_floor(liquidity_x64, delta, 1u128 << 64)?;

        // multiplying Q64.64 scales to Q128.128 because (2^64 * 2^64) = 2^128
        // we divide with Q64.64 to scale it back to Q64.64 because
        // (2^128 / 2^64) = 2^64
        let denominator = mul_div_floor(sqrt_ratio_b_x64, sqrt_ratio_a_x64, 1u128 << 64)?;

        // numerator - Q64.64
        // denominator - Q64.64
        // (A^64 / B^64) = A / B
        // so result is in Q0.0
        Ok(numerator
            .checked_div(denominator)
            .ok_or(MathError::ZeroDenominator)?)
    }

    /// Calculate how many token_1 user will get given liquidity, lower and upper bound
    /// # Formula
    /// y = L * (√P_upper - √P_lower)
    ///
    /// # Arguments
    /// `sqrt_ratio_a_x64` - lower tick bound in Q64.64
    /// `sqrt_ratio_b_x64` - upper tick bound in Q64.64
    /// `liquidity` - liquidity in given tick range in Q0.0
    ///
    /// Throws if liquidity = 0
    ///
    /// # Return
    /// token_1 in Q0.0
    pub fn get_amount_1_for_liquidity(
        mut sqrt_ratio_a_x64: u128,
        mut sqrt_ratio_b_x64: u128,
        liquidity: u128,
    ) -> Result<u128, MathError> {
        if liquidity == 0 {
            return Err(MathError::ZeroLiquidity);
        }

        if sqrt_ratio_a_x64 > sqrt_ratio_b_x64 {
            std::mem::swap(&mut sqrt_ratio_a_x64, &mut sqrt_ratio_b_x64);
        }

        let delta = sqrt_ratio_b_x64
            .checked_sub(sqrt_ratio_a_x64)
            .ok_or(MathError::Overflow)?;

        // liquidity is in Q0.0
        // delta is in Q64.64
        // multiplication will scale it to Q64.64
        // divide 2^64 / 2^64 to cast back to Q0.0
        Ok(mul_div_floor(liquidity, delta, 1u128 << 64))?
    }

    /// Given lower, upper bound current pool price and liquidity claculates how many token_x and token_y
    /// user will get
    ///
    /// 1. if pool price <= lower bound then return (token_x amount 0)
    /// 2. if pool price >= upper bound return (0 token_y amount)
    /// 3. if pool price is in between then return (token_x token_y)
    ///
    /// # Arguments
    /// `sqrt_ratio_a_x64` - lower tick bound in Q64.64
    /// `sqrt_ratio_b_x64` - upper tick bound in Q64.64
    /// `sqrt_ratio_x64` - current pool price in Q64.64
    /// `liquidity` - liquidity in given tick range in Q0.0
    ///
    /// Throws if liquidity = 0 or sqrt_ratio_x64 = 0
    ///
    /// # Returns
    /// TokenAccounts{amount_0, amount_1}
    ///
    /// # Note one of the amount can be zero based on the condition so make sure to verify it
    pub fn get_amounts_for_liquidity(
        mut sqrt_ratio_a_x64: u128,
        mut sqrt_ratio_b_x64: u128,
        sqrt_ratio_x64: u128,
        liquidity: u128,
    ) -> Result<TokenAmounts, MathError> {
        if liquidity == 0 {
            return Err(MathError::ZeroLiquidity);
        }

        if sqrt_ratio_x64 == 0 {
            return Err(MathError::ZeroSqrtPrice);
        }

        if sqrt_ratio_a_x64 > sqrt_ratio_b_x64 {
            std::mem::swap(&mut sqrt_ratio_a_x64, &mut sqrt_ratio_b_x64);
        }

        if sqrt_ratio_x64 < sqrt_ratio_a_x64 {
            // only token_0 is active
            let amount_0 =
                Self::get_amount_0_for_liquidity(sqrt_ratio_a_x64, sqrt_ratio_b_x64, liquidity)?;

            Ok(TokenAmounts {
                amount_0,
                amount_1: 0,
            })
        } else if sqrt_ratio_x64 <= sqrt_ratio_b_x64 {
            // if we are here we know Pc > Pb
            // and this branch checks Pc < Pb
            // Pa < Pc < Pa

            let amount_0 =
                Self::get_amount_0_for_liquidity(sqrt_ratio_x64, sqrt_ratio_b_x64, liquidity)?;
            let amount_1 =
                Self::get_amount_1_for_liquidity(sqrt_ratio_a_x64, sqrt_ratio_x64, liquidity)?;

            Ok(TokenAmounts { amount_0, amount_1 })
        } else {
            // we know Pa > Pc and Pb < Pc
            // so only token_1 is active
            let amount_1 =
                Self::get_amount_1_for_liquidity(sqrt_ratio_a_x64, sqrt_ratio_b_x64, liquidity)?;

            Ok(TokenAmounts {
                amount_0: 0,
                amount_1,
            })
        }
    }
}

#[cfg(test)]
mod liquidity_test {

    use crate::tick_math::{MAX_SQRT_PRICE_X64, MIN_SQRT_PRICE_X64};

    use super::*;

    use proptest::prelude::*;

    proptest! {
        #[test]
        fn fuzz_test_liquidity_round_trip(
            amount_0 in 0u64..1_000_000_000,
            amount_1 in 0u64..1_000_000_000,
            sqrt_a in MIN_SQRT_PRICE_X64..MAX_SQRT_PRICE_X64,
            sqrt_b in MIN_SQRT_PRICE_X64..MAX_SQRT_PRICE_X64,
            sqrt_c in MIN_SQRT_PRICE_X64..MAX_SQRT_PRICE_X64
        ) {
            let sqrt_pa = u128::min(sqrt_a, sqrt_b);
            let sqrt_pb = u128::max(sqrt_a, sqrt_b);

            if sqrt_pa == sqrt_pb {
                return Ok(())
            }

            let liquidity = LiquidityMath::get_liquidity_from_amounts(
                sqrt_c, sqrt_pa, sqrt_pb, amount_0, amount_1).unwrap();

            if liquidity == 0 {
                return Ok(())
            }

            let returned = LiquidityMath::get_amounts_for_liquidity(
                sqrt_pa, sqrt_pb, sqrt_c, liquidity).unwrap();


            assert!(
                returned.amount_0 <= u128::from(amount_0),
                "Returned amount_0 ({}) exceeded max amount_0 ({})",
                returned.amount_0,
                amount_0
            );

            assert!(
                returned.amount_1 <= u128::from(amount_1),
                "Returned amount_1 ({}) exceeded max amount_1 ({})",
                returned.amount_1,
                amount_1
            );

        }
    }
}
