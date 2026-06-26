use crate::{error::MathError, full_math::mul_shift_64};

/// Minimum valid tick
pub const MIN_TICK: i32 = -443636;
/// Maximum valid tick
pub const MAX_TICK: i32 = 443636;
/*
 * why BIT_PRECISION 14 is sufficient
 * we want our error tolerance be < 1.0 why because if its < 1.0
 * because integer will floor so for example if we have 99.442, 99.29299, 99.99 they
 * will truncate towards 99 why we want this because then we can call our
 * forward function tick_to_sqrt_price_x64(raw_tick + 1) so if actual price tick is 100
 * and our algorithm calculates something like 99.329 and it will truncate to 99
 * we can do tick_to_sqrt_price_x64(100) and since actual price is at tick 100 the condition
 * sqrt_price_x64 >= sqrt_price_next_tick will pass and we will return 0
 *
 * proof that max error will be < 1.0 for BIT_PRECISION 14
 * we define max error = 2^-BIT_PRECISION = 2^-14 = 1/16384
 * 1/16384 = 0.000061035
 *
 * to convert log2 error into tick error
 * K = 2/log2(1.0001) = 13863.019
 *
 * max tick error = max log2 error * k max tick error
 * 0.000061035 - 13863.019 = `0.846`
 * and since 0.846 < 1.0 we have mathematical proof that tick will never off by 0.846
 *
 * the invariant is: 2^-n * K < 1.0 where n is bit precision
 */
const BIT_PRECISION: u32 = 14;

/// calculated using #tick_to_sqrt_price_x64(MIN_TICK)
pub const MIN_SQRT_PRICE_X64: u128 = 4295048016;
/// calculated using #tick_to_sqrt_price_x64(MAX_TICK)
pub const MAX_SQRT_PRICE_X64: u128 = 79226673515401279963822778343;

pub struct TickMath;

impl TickMath {
    /// Given tick calculate sqrt_price in Q64.64 format
    /// uses binary decomposition algorithm and magic constants calculated off chain for precision
    /// example: if tick is 13 then the function breaks it into 8 + 4 + 1
    /// which is 2^3 + 2^2 + 2^0
    /// and based on set bit it multiply with pre-calculated constants FACTORS 1.0001^(2^i / 2) * 2^64 for positive ticks and INV_FACTOR = 2^64 / (1.0001^(2^i / 2)) for negative ticks where
    /// i = 0..18
    /// # if tick is positive it calls private function `get_sqrt_price_positive_tick` else it calls
    /// `get_sqrt_price_negative_tick`
    /// in case of tick = 13 it does Factors[3] * Factors[2] + Factors[0] done
    /// instead of multiplying it 13 times but the main point of this alogorithm is to avoid precsion loss
    /// because 1.0001^tick/2 is cumbersome to calculate in on-chain
    ///
    /// will throw error if |tick| > 443636 or -tick < -443636
    ///
    /// # Arguments
    /// * `tick` i32
    pub fn tick_to_sqrt_price_x64(tick: i32) -> Result<u128, MathError> {
        if tick < MIN_TICK || tick > MAX_TICK {
            return Err(MathError::InvalidTickRange);
        }

        if tick >= 0 {
            Ok(Self::get_sqrt_price_positive_tick(tick))?
        } else {
            Ok(Self::get_sqrt_price_negative_tick(tick))?
        }
    }

    fn get_sqrt_price_positive_tick(tick: i32) -> Result<u128, MathError> {
        let mut ratio = if tick & 1 != 0 {
            18447666387855959850
        } else {
            1u128 << 64
        };

        // i = 1
        if (tick & 2) != 0 {
            ratio = mul_shift_64(ratio, 18448588748116922571);
        }

        // i = 2
        if (tick & 4) != 0 {
            ratio = mul_shift_64(ratio, 18450433606991734263);
        }

        // i = 3
        if (tick & 8) != 0 {
            ratio = mul_shift_64(ratio, 18454123878217468680);
        }

        // i = 4
        if (tick & 16) != 0 {
            ratio = mul_shift_64(ratio, 18461506635090006701);
        }

        // i = 5
        if (tick & 32) != 0 {
            ratio = mul_shift_64(ratio, 18476281010653910144);
        }

        // i = 6
        if (tick & 64) != 0 {
            ratio = mul_shift_64(ratio, 18505865242158250041);
        }

        // i = 7
        if (tick & 128) != 0 {
            ratio = mul_shift_64(ratio, 18565175891880433522);
        }

        // i = 8
        if (tick & 256) != 0 {
            ratio = mul_shift_64(ratio, 18684368066214940582);
        }

        // i = 9
        if (tick & 512) != 0 {
            ratio = mul_shift_64(ratio, 18925053041275764671);
        }

        // i = 10
        if (tick & 1024) != 0 {
            ratio = mul_shift_64(ratio, 19415764168677886926);
        }

        // i = 11
        if (tick & 2048) != 0 {
            ratio = mul_shift_64(ratio, 20435687552633177494);
        }

        // i = 12
        if (tick & 4096) != 0 {
            ratio = mul_shift_64(ratio, 22639080592224303007);
        }

        // i = 13
        if (tick & 8192) != 0 {
            ratio = mul_shift_64(ratio, 27784196929998399742);
        }

        // i = 14
        if (tick & 16384) != 0 {
            ratio = mul_shift_64(ratio, 41848122137994986128);
        }

        // i = 15
        if (tick & 32768) != 0 {
            ratio = mul_shift_64(ratio, 94936283578220370716);
        }

        // i = 16
        if (tick & 65536) != 0 {
            ratio = mul_shift_64(ratio, 488590176327622479860);
        }

        // i = 17
        if (tick & 131072) != 0 {
            ratio = mul_shift_64(ratio, 12941056668319229769860);
        }

        // i = 18
        if (tick & 262144) != 0 {
            ratio = mul_shift_64(ratio, 9078618265828848800676189);
        }

        Ok(ratio)
    }

    fn get_sqrt_price_negative_tick(tick: i32) -> Result<u128, MathError> {
        // tick is negative so make it positive for bit checks
        let tick_abs = tick.abs();

        let mut ratio = if tick_abs & 1 != 0 {
            18445821805675395072
        } else {
            1u128 << 64
        };

        // i = 1
        if (tick_abs & 2) != 0 {
            ratio = mul_shift_64(ratio, 18444899583751176192);
        }

        // i = 2
        if (tick_abs & 4) != 0 {
            ratio = mul_shift_64(ratio, 18443055278223355904);
        }

        // i = 3
        if (tick_abs & 8) != 0 {
            ratio = mul_shift_64(ratio, 18439367220385607680);
        }

        // i = 4
        if (tick_abs & 16) != 0 {
            ratio = mul_shift_64(ratio, 18431993317065451520);
        }

        // i = 5
        if (tick_abs & 32) != 0 {
            ratio = mul_shift_64(ratio, 18417254355718166528);
        }

        // i = 6
        if (tick_abs & 64) != 0 {
            ratio = mul_shift_64(ratio, 18387811781193596928);
        }

        // i = 7
        if (tick_abs & 128) != 0 {
            ratio = mul_shift_64(ratio, 18329067761203533824);
        }

        // i = 8
        if (tick_abs & 256) != 0 {
            ratio = mul_shift_64(ratio, 18212142134806112256);
        }

        // i = 9
        if (tick_abs & 512) != 0 {
            ratio = mul_shift_64(ratio, 17980523815641602048);
        }

        // i = 10
        if (tick_abs & 1024) != 0 {
            ratio = mul_shift_64(ratio, 17526086738831245312);
        }

        // i = 11
        if (tick_abs & 2048) != 0 {
            ratio = mul_shift_64(ratio, 16651378430235211776);
        }

        // i = 12
        if (tick_abs & 4096) != 0 {
            ratio = mul_shift_64(ratio, 15030750278693769216);
        }

        // i = 13
        if (tick_abs & 8192) != 0 {
            ratio = mul_shift_64(ratio, 12247334978883385344);
        }

        // i = 14
        if (tick_abs & 16384) != 0 {
            ratio = mul_shift_64(ratio, 8131365268885459968);
        }

        // i = 15
        if (tick_abs & 32768) != 0 {
            ratio = mul_shift_64(ratio, 3584323654723988992);
        }

        // i = 16
        if (tick_abs & 65536) != 0 {
            ratio = mul_shift_64(ratio, 696457651847846528);
        }

        // i = 17
        if (tick_abs & 131072) != 0 {
            ratio = mul_shift_64(ratio, 26294789957471032);
        }

        // i = 18
        if (tick_abs & 262144) != 0 {
            ratio = mul_shift_64(ratio, 37481735321136);
        }

        Ok(ratio)
    }

    /// Given sqrt_price in Q64.64 find tick
    /// # Formula
    /// T = log2(√P) * (2/log2(1.0001))
    /// # Arguments
    /// `sqrt_price_x64`: u128 (sqrt_price in Q64.64).
    ///
    /// Throws if sqrt_price_x64 >= MIN_SQRT_PRICE_X64 or sqrt_price_x64 > MAX_SQRT_PRICE_X64
    pub fn sqrt_price_x64_to_tick(sqrt_price_x64: u128) -> Result<i32, MathError> {
        if sqrt_price_x64 <= MIN_SQRT_PRICE_X64 || sqrt_price_x64 > MAX_SQRT_PRICE_X64 {
            return Err(MathError::SqrtPriceOutOfBounds);
        }

        /*
         * counting from left side return how many zeros is present in binary without counting set bit
         * for example consider 0b0001000 (8 in binary)
         * couting from left it will return 4 zeros
         * now to get msb we do BIT_SIZE - LZ - 1
         * here we we have 8 bit
         * so msb = 8 - 4 - 1 = 3
         * so msb = 3 now here is intresting part
         * for numbers whose bit is set we can write it as 2^i this is fundamental rule of binary
         * so here is the magic:
         * 2^3 = 8 and log2(8) = 3, msb == log2
         */
        let msb = 128 - sqrt_price_x64.leading_zeros() - 1;

        // integer part of log2 thanks to msb we get this for almost free
        // -64 because remember sqrt_price_x64 is in Q64.64 we have to scale it back to Q0.0
        let log2p_integer_x14 = (msb as i128 - 64) << BIT_PRECISION;

        // so at this point we have log2 integer part let's call it y
        // now we have to find log2(m) where m is fractional part

        /*
         * any positive integer can be written in form of 2^integer * m
         * where integer is log2 integer part and m is mantissa b/w [1.0, 2.0)
         * we have 2^integer part already now we have to find fractional part
         * extract the fractional part and normalize m b/w [1.0, 2.0)
         */
        let mut m: u128 = if msb >= 64 {
            sqrt_price_x64 >> (msb - 64)
        } else {
            sqrt_price_x64 << (64 - msb)
        };

        // to store the fractional bits
        let mut frac_log2: u128 = 0;

        /*
         * core alogorithm:
         * log2(x) = integer + 0.b1*b2*b3*b4...
         * where each bi is digit (either 0 or 1)
         *
         * how fractions are represented in binary
         * the integer part is represented as bi * 2^3 + bi * 2^2 + bi * 2^1 + bi * 2^0
         * the fractional part is represented as bi * 2^-1 + bi * 2^-2 + bi * 2^-3 ....
         *
         * calculate m^2
         * if m^2 >= 2.0 then set bit to 1 and normalize m again to 1.0 <= m < 2.0
         * else skip it
         *
         * why?
         * if log2(m) >= 1/2 (in decimal) then bi = 1 else bi = 0
         * since m is bounded by 1.0 <= m < 2.0 its logarithm is bounded by 0.0 <= log2(m) < 1.0
         *
         * now why >= 2.0 and not something else
         * log2(m) >= 1/2
         * = 2^log2(m) >= 2^1/2
         * = m >= √2
         * = `m^2 >= 2`
         *
         * that's why if m^2 >= 2 then m >= √2, which means log2(m) >= 0.5, so bi = 1
         * if m^2 < 2 then m < √2, which means log2(m) < 0.5, so bi = 0
         */
        for i in 0..BIT_PRECISION {
            let mut new_m = mul_shift_64(m, m);

            // 1u128 << 65 = 2.0 in Q64.64
            if new_m >= (1u128 << 65) {
                // set the bit to 1
                frac_log2 |= 1u128 << (BIT_PRECISION - 1 - i);
                // normalize new_m back to 1 <= m < 2
                new_m >>= 1;
            }

            m = new_m;
        }

        // combine y + log2(m)
        let log2p_x14 = log2p_integer_x14 + frac_log2 as i128;

        /*
         * at this point we have full approx log2 now how can we find tick
         * the formula is:
         * T = formula: T = log2(√p) * (2/log2(1.0001))
         */

        // calculated using
        // (2 / math.log2(1.0001)) * 2^32
        const LOG_B_2_X32: i128 = 59543866431248i128;

        // transform from base2 to base 1.0001
        let mut log_sqrt_1001_x46 = log2p_x14 * LOG_B_2_X32;

        if log_sqrt_1001_x46 < 0 {
            log_sqrt_1001_x46 += (1i128 << 46) - 1;
        }

        // scale back from Q46.46 to Q0.0
        log_sqrt_1001_x46 >>= 46;

        let tick = log_sqrt_1001_x46 as i32;

        /*
         * at this point we have approx tick because our Bit precision was 14
         * the error tolerance `0.846` so if our actual tick was 100 then at this point
         * we have ~99.154 which is truncated to 99 in integer
         * so we have to call a forward function `tick_to_sqrt_price_x64` with passing tick + 1 to
         * get actual tick.
         * and if sqrt_price_x64 >= actual_tick_price then return tick + 1
         * else return tick
         */

        let actual_tick_high_sqrt_price = Self::tick_to_sqrt_price_x64(tick + 1)?;

        if sqrt_price_x64 >= actual_tick_high_sqrt_price {
            Ok(tick + 1)
        } else {
            Ok(tick)
        }
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    #[test]
    fn test_tick_zero() {
        let sqrt_price = TickMath::tick_to_sqrt_price_x64(0).unwrap();
        assert_eq!(sqrt_price, 1u128 << 64);
    }

    #[test]
    fn test_min_max_tick() {
        let _min: u128 = TickMath::tick_to_sqrt_price_x64(MIN_TICK).unwrap();
        let _max: u128 = TickMath::tick_to_sqrt_price_x64(MAX_TICK).unwrap();
    }

    #[test]
    fn test_round_trip() {
        let sqrt_price = TickMath::tick_to_sqrt_price_x64(100).unwrap();
        let tick = TickMath::sqrt_price_x64_to_tick(sqrt_price).unwrap();

        assert_eq!(tick, 100);

        let sqrt_price = TickMath::tick_to_sqrt_price_x64(-121).unwrap();
        let tick = TickMath::sqrt_price_x64_to_tick(sqrt_price).unwrap();

        assert_eq!(tick, -121);
    }

    proptest! {
        #[test]
        fn fuzz_round_trip_tick_to_sqrt_and_back(tick in -MIN_TICK..=MAX_TICK) {
          let sqrt_price = TickMath::tick_to_sqrt_price_x64(tick).unwrap();

          let result_tick = TickMath::sqrt_price_x64_to_tick(sqrt_price).unwrap();

          prop_assert_eq!(result_tick, tick);

        }
    }
}
