pub struct BitmapMath;

impl BitmapMath {
    /*
     * consider this array
     * [0, 0, 0, 0, 0, .............. 1, 0, 1] (63 bits)
     * [1, 0, 1, 0, 0, .............. 0, 0, 1] (63 bits)
     * [0, 0, 1, 0, 0, .............. 0, 1, 1] (63 bits)
     * [0, 1, 0, 0, 1, .............. 0, 0, 1] (63 bits)

     each row is 64 bits long (call this word)
     each bit in row have a state (call this bit)

     if bit == 1 then tick is initialized else tick is not initialized
     bit can only have two states either 0 or 1

     our job is to find the next initialized tick without looping over whole array O(N)

     here is the thing in protocol not every tick will be initialized
     tick's can only be initialized if they are divisible by `tick_spacing`

     so we have compressed = tick_index / tick_spacing

    fundamental operatation:
    word_index = compressed_tick >> 6 (compressed_tick / 64)
    bit_index = compress_tick & 63

    example:
    consider compressed_tick = 20
    20 >> 6 = 1
    20 % 64 = 20

    so our current tick is row[1] bit 20

    [00000000010001...000000]
              | curr tick

    now we have to find next tick look at the bit we know next init tick is at bit let's say bit 24
    but we can't just jump to bit 24

    so here is idea we have  `trailing_zeros()` it return the count of zero from right side (LSB)
    ex: [00101000] counting from right (LSB) we have `3` zeros before 1 so `trailing_zeros()` will return 3

    so what if we take advantage of this if we mask our current tick and lower bits then the `trailing_zeros()`
    will return zero's count until next tick we can use this information to find next tick
    efficiently

    first how do we set the current tick and it's lower bits to 0 ?

    we create mask for this let's say our word is 8 bits long [01010101]

    now consider current tick is 5 and next initialized tick is 7 (counting from zero)
    01010000
     | |
       curr tick

    current as we know `trailing_zeros()` will return 4 and next bit is current tick but we want next
    initialized tick

    how about we set 00001 to 00000
    create a initial mask 11111111

    now shift initial mask to (current_bit + 1) (+1 because we also want to mask current bit)

    in our case 11111111 << 5

    mask = 11100000 this is our mask

    now we will do AND operation with current row and mask

      01010000 (row)
    & 11100000 (mask)
    ----------
      01000000

      now if we do `trailing_zeros()` it will return 6 and we know the next initialized tick

      so in short
      take the current bit + lower bits set it to 0
      call trailing_zeros() if the next bit is set in the row it will return
      either go to next row


      so this is for (a_to_b = false) when user is swapping token y for token x and price is going up so tick is
      index goes up

      so when user swaps token x for token y the price goes down so does tick index will go down
      so in that case we have to find lower tick

      here is how it looks i am considering my word is 8 bit long

      current tick is at bit 6 next lower tick is at bit 3
      01001010
       |

     we have to find the next lower initialized tick

     how about we set the current tick bit and its upper bit to 0 then when we call `leading_zeros()`
     it will count zeros from MSB (left side or upper to lower)

     to mask current_bit and its upper bits
     we can do (1u8 << current_bit) - 1

     1 = 00000001
     current_bit = 6

     = 01000000 - 1

     mask = 00111111

     now we will do

      01001010 (word)
    & 00111111 (mask)
      --------
      00001000 (masked_row)

      calling `leading_zeros()` on masked_row will return `4` but our next init tick is 3
      so we will do (WORD_BIT - 1) - leading_zeros()
      = (8 - 1) - 4
      = 3 (our next lower initialized tick)

      summary
      masked the current tick bit and its upper bits
      calling leading_zeros() on masked row
      use (WORD_BIT - 1) - leading_zeros() to find next lower tick

     */

    pub fn next_initialized_tick(bitmap: &[u64], tick: i32, tick_spacing: i32) -> Option<i32> {
        let compressed = if tick >= 0 {
            tick / tick_spacing as i32
        } else {
            (tick - tick_spacing as i32 + 1) / tick_spacing as i32
        };

        let mut word_idx = compressed >> 6;
        let bit_idx: u32 = (compressed & 63) as u32;

        if word_idx < 0 {
            return None;
        }

        let mut mask = u64::MAX.checked_shl(bit_idx + 1).unwrap_or(0);

        while (word_idx as usize) < bitmap.len() {
            let masked = bitmap[word_idx as usize] & mask;

            if masked != 0 {
                let next_bit = masked.trailing_zeros();
                let compressed = (word_idx * 64) + next_bit as i32;
                return Some(compressed * tick_spacing);
            } else {
                word_idx += 1;
                // search the next word from start
                mask = u64::MAX;
            }
        }

        None
    }

    pub fn previous_initialized_tick(bitmap: &[u64], tick: i32, tick_spacing: i32) -> Option<i32> {
        let compressed = if tick >= 0 {
            tick / tick_spacing as i32
        } else {
            (tick - tick_spacing as i32 + 1) / tick_spacing as i32
        };

        let mut word_idx = compressed >> 6;
        let bit_idx: u32 = (compressed & 63) as u32;

        if word_idx < 0 {
            return None;
        }

        let mut mask = (1u64 << bit_idx) - 1;

        while word_idx >= 0 && (word_idx as usize) < bitmap.len() {
            let masked = bitmap[word_idx as usize] & mask;

            if masked != 0 {
                let prev_bit = 63 - masked.leading_zeros();
                let compressed = (word_idx * 64) + prev_bit as i32;
                return Some(compressed * tick_spacing);
            } else {
                word_idx -= 1;
                // search the next word from start
                mask = u64::MAX;
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn fuzz_next_and_previous_initialized_tick(
            word0 in 0u64..u64::MAX,
            word1 in 0u64..u64::MAX,
            word2 in 0u64..u64::MAX,
            word3 in 0u64..u64::MAX,
            current_tick in 0i32..255, // Covers our 4 words (256 bits)
        ) {
            let bitmap = [word0, word1, word2, word3];
            let tick_spacing = 1; // Keep spacing 1 for simple brute-force comparison

            // --- Test next_initialized_tick ---
            let mut expected_next: Option<i32> = None;
            // Brute force search forward
            for t in (current_tick + 1)..256 {
                let word_idx = (t / 64) as usize;
                let bit_idx = (t % 64) as u32;
                if (bitmap[word_idx] & (1u64 << bit_idx)) != 0 {
                    expected_next = Some(t);
                    break;
                }
            }

            let result_next = BitmapMath::next_initialized_tick(&bitmap, current_tick, tick_spacing);
            prop_assert_eq!(result_next, expected_next, "Next tick failed");

            // --- Test previous_initialized_tick ---
            let mut expected_prev: Option<i32> = None;
            // Brute force search backward
            for t in (0..current_tick).rev() {
                let word_idx = (t / 64) as usize;
                let bit_idx = (t % 64) as u32;
                if (bitmap[word_idx] & (1u64 << bit_idx)) != 0 {
                    expected_prev = Some(t);
                    break;
                }
            }
            let result_prev = BitmapMath::previous_initialized_tick(&bitmap, current_tick, tick_spacing);
            prop_assert_eq!(result_prev, expected_prev, "Previous tick failed");
        }
    }
}
