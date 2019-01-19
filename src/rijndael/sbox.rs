use crate::galois;
use crate::galois::Gf256;
use lazy_static::lazy_static;

fn inv(n: Gf256) -> u8 {
    if n == Gf256(0) {
        0
    } else {
        u8::from(galois::inv(n))
    }
}

pub struct Sbox {
    direct: [Gf256; 256],
    inverse: [Gf256; 256],
}

lazy_static! {
    pub static ref SBOX: Sbox = {
        let mut direct = [Default::default(); 256];
        let mut inverse = [Default::default(); 256];

        for i in 0..=255 {
            let n = Gf256(i);
            let mut s = inv(n);
            let mut x = s;

            for _ in 0..4 {
                s = s.rotate_left(1);
                x ^= s;
            }

            x ^= 99;

            direct[i as usize] = Gf256(x);
            inverse[x as usize] = Gf256(i);
        }

        Sbox { direct, inverse }
    };
}

impl Sbox {
    pub fn direct(&self, Gf256(n): Gf256) -> Gf256 {
        self.direct[n as usize]
    }

    #[allow(dead_code)]
    pub fn inverse(&self, Gf256(n): Gf256) -> Gf256 {
        self.inverse[n as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sbox() {
        assert_eq!(Gf256(0x63), SBOX.direct(Gf256(0x00)));
        assert_eq!(Gf256(0xda), SBOX.direct(Gf256(0x7a)));
    }
}
