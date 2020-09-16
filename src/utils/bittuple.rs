//! Bit twiddling helpers

pub fn to_bits(b: u8) -> (u8,u8,u8,u8,u8,u8,u8,u8) {
    ((b >> 7) & 1, (b >> 6) & 1, (b >> 5) & 1, (b >> 4) & 1,
     (b >> 3) & 1, (b >> 2) & 1, (b >> 1) & 1, (b >> 0) & 1)
}

pub fn to_u32(bits: &[u8]) -> u32 {
    let mut result: u32 = 0;
    for b in bits {
        result = result * 2 + (*b as u32);
    }
    result
}