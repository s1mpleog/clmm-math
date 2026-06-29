pub struct BitmapMath;

impl BitmapMath {
    pub fn next_initialized_tick() {
        /*
         * consider this array
         * [0, 0, 0, 0, 0, .............. 1, 0, 1] (64 bits)
         * [1, 0, 1, 0, 0, .............. 0, 0, 1] (64 bits)
         * [0, 0, 1, 0, 0, .............. 0, 1, 1] (64 bits)
         * [0, 1, 0, 0, 1, .............. 0, 0, 1] (64 bits)

         each row is 64 bits long (call this word)
         each bit in row have a state (call this bit)

         if bit == 1 then tick is initialized else tick is not initialized
         bit can only have two states either 0 or 1

         our job is to find the next initialized tick without looping over whole array O(N)

         here is the thing in protocol not every tick will be initialized
         tick's can only be initialized if they are divisible by `tick_spacing`

         so we have compressed = tick_index / tick_spacing

        fundamental operatation:
        word_index = compressed_tick >> 6 (compressed_tick / 2^64)
        bit_index = compress_tick & 63 (counting 64 from 0)



         */

        unimplemented!()
    }
}
