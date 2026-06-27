use crate::error::MathError;

pub struct SwapMath;

pub const FEE_DENOMINATOR: u32 = 1_000_000;

#[derive(Debug, Default)]
pub struct SwapStepResult {
    pub sqrt_price_next_x64: u128,
    pub amount_in: u64,
    pub amount_out: u64,
    pub fee_amount: u64,
}

impl SwapMath {
    fn get_amount_0_delta() {}

    fn get_amount_1_delta() {}

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

        let result = SwapStepResult::default();

        unimplemented!()
    }
}
