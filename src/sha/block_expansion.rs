use byteorder::{BigEndian, ReadBytesExt};
use std::io::Cursor;

fn sigma0(x: u32) -> u32 {
    x.rotate_right(7) ^ x.rotate_right(18) ^ x >> 3
}

fn sigma1(x: u32) -> u32 {
    x.rotate_right(17) ^ x.rotate_right(19) ^ x >> 10
}

pub fn expand_block(block: &[u8], pad: &mut [u32; 64]) {
    assert_eq!(64, block.len());

    let mut rdr = Cursor::new(&block[0..64]);
    rdr.read_u32_into::<BigEndian>(&mut pad[0..16]).unwrap();

    for i in 16..64 {
        pad[i] = sigma1(pad[i - 2])
            .wrapping_add(pad[i - 7])
            .wrapping_add(sigma0(pad[i - 15]))
            .wrapping_add(pad[i - 16]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sigma0() {
        assert_eq!(0b1000_1000_0000_0001, sigma0(0b100_0000_0000_0000_0000));
        assert_eq!(0b10_0000_0000_0100_0000_0000_0000, sigma0(1));
    }

    #[test]
    fn test_sigma1() {
        assert_eq!(0b10_0000_0101, sigma1(0b1000_0000_0000_0000_0000));
        assert_eq!(0b1010_0000_0000_0000, sigma1(1));
    }
}
