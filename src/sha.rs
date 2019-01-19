use byteorder::{BigEndian, WriteBytesExt};
use crate::prime;
use lazy_static::lazy_static;

mod block_expansion;
mod padding;

fn ch(x: u32, y: u32, z: u32) -> u32 {
    (x & y) ^ (!x & z)
}

fn maj(x: u32, y: u32, z: u32) -> u32 {
    (x & y) ^ (x & z) ^ (y & z)
}

fn sigma0(x: u32) -> u32 {
    x.rotate_right(2) ^ x.rotate_right(13) ^ x.rotate_right(22)
}

fn sigma1(x: u32) -> u32 {
    x.rotate_right(6) ^ x.rotate_right(11) ^ x.rotate_right(25)
}

fn fpart32(x: f64) -> u32 {
    (x.fract() * 32_f64.exp2()).trunc() as u32
}

fn to_fpart32(mut f: impl FnMut(f64) -> f64) -> impl FnMut(u32) -> u32 {
    move |p| fpart32(f(f64::from(p)))
}

struct Tables {
    round_constants: Vec<u32>,
    init_hash: Vec<u32>,
}

lazy_static! {
    static ref TABLES: Tables = {
        let mut primes = prime::Primes::new();
        let round_constants: Vec<u32> = primes
            .into_iter()
            .take(64)
            .map(to_fpart32(&f64::cbrt))
            .collect();
        let init_hash: Vec<u32> = primes
            .into_iter()
            .take(8)
            .map(to_fpart32(&f64::sqrt))
            .collect();
        Tables {
            round_constants,
            init_hash,
        }
    };
}

pub struct Sha256<'a> {
    message: Vec<u8>,
    length: usize,
    hash: Hasher<'a>,
}

pub struct Hasher<'a> {
    hash: [u32; 8],
    tables: &'a Tables,
    round_keys: [u32; 64],
}

impl Sha256<'_> {
    pub fn new<'a>() -> Sha256<'a> {
        let tables = &*TABLES;
        let mut res = Sha256 {
            message: Vec::with_capacity(128),
            length: 0,
            hash: Hasher {
                tables,
                hash: [0; 8],
                round_keys: [0; 64],
            },
        };
        res.hash.reset();
        res
    }

    pub fn hash<'a, T: IntoIterator<Item = &'a u8>>(message: T) -> Vec<u8> {
        Self::new().extend(message).flush()
    }

    pub fn extend<'a, T: IntoIterator<Item = &'a u8>>(&mut self, message: T) -> &mut Self {
        let mut iter = message.into_iter();
        loop {
            let l = self.message.len();
            self.message.extend(iter.by_ref().take(64 - l));
            self.length += self.message.len() - l;
            if self.message.len() < 64 {
                return self;
            }
            self.hash.hash_block(&self.message);
            self.message.clear();
        }
    }

    fn hash_message_remainder(&mut self) {
        padding::pad_448(&mut self.message);
        self.message
            .write_u64::<BigEndian>((self.length * 8) as u64)
            .unwrap();
        for block in self.message.chunks(64) {
            self.hash.hash_block(block);
        }
    }

    pub fn flush(&mut self) -> Vec<u8> {
        self.hash_message_remainder();
        self.message.clear();
        self.hash.reset()
    }
}

impl Default for Sha256<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl Hasher<'_> {
    fn hash_block(&mut self, message: &[u8]) {
        assert_eq!(64, message.len());
        block_expansion::expand_block(message, &mut self.round_keys);
        let mut r = self.hash;
        for (&ki, &wi) in self
            .tables
            .round_constants
            .iter()
            .zip(self.round_keys[..].iter())
        {
            let t1 = r[7]
                .wrapping_add(sigma1(r[4]))
                .wrapping_add(ch(r[4], r[5], r[6]))
                .wrapping_add(ki)
                .wrapping_add(wi);
            let t2 = sigma0(r[0]).wrapping_add(maj(r[0], r[1], r[2]));
            r[7] = r[6];
            r[6] = r[5];
            r[5] = r[4];
            r[4] = r[3].wrapping_add(t1);
            r[3] = r[2];
            r[2] = r[1];
            r[1] = r[0];
            r[0] = t1.wrapping_add(t2);
        }
        for (h, &r) in self.hash.iter_mut().zip(&r) {
            *h = h.wrapping_add(r);
        }
    }

    fn reset(&mut self) -> Vec<u8> {
        let mut res = Vec::with_capacity(32);
        for (&ih, h) in self.tables.init_hash.iter().zip(&mut self.hash) {
            res.write_u32::<BigEndian>(*h).unwrap();
            *h = ih;
        }
        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::iter;

    #[test]
    fn test_ch() {
        assert_eq!(0, ch(0, 0, 0));
        assert_eq!(0xf0, ch(0x00, 0xff, 0xf0));
        assert_eq!(0xa5, ch(0xff, 0xa5, 0xf7));
        assert_eq!(0xa7, ch(0xf0, 0xa5, 0xf7));
    }

    #[test]
    fn test_maj() {
        assert_eq!(0x00, maj(0x00, 0x00, 0x00));
        assert_eq!(0x00, maj(0x0f, 0x00, 0x00));
        assert_eq!(0x00, maj(0x00, 0x0f, 0x00));
        assert_eq!(0x00, maj(0x00, 0x00, 0x0f));
        assert_eq!(0x0f, maj(0x0f, 0x0f, 0x00));
        assert_eq!(0x0f, maj(0x00, 0x0f, 0x0f));
        assert_eq!(0x0f, maj(0x0f, 0x00, 0x0f));
        assert_eq!(0x0f, maj(0x0f, 0x0f, 0x0f));
    }

    #[test]
    fn test_sigma0() {
        assert_eq!(
            0b1_0000_0000_0010_0000_0001,
            sigma0(0b100_0000_0000_0000_0000_0000)
        );
    }

    #[test]
    fn test_sigma1() {
        assert_eq!(
            0b1000_0100_0000_0000_0001,
            sigma1(0b10_0000_0000_0000_0000_0000_0000)
        );
    }

    #[test]
    fn test_round_constants() {
        assert_eq!(0x428a2f98, TABLES.round_constants[0]);
        assert_eq!(0x71374491, TABLES.round_constants[1]);
        assert_eq!(0xb5c0fbcf, TABLES.round_constants[2]);
        assert_eq!(0xe9b5dba5, TABLES.round_constants[3]);
        assert_eq!(0x3956c25b, TABLES.round_constants[4]);
        assert_eq!(0x59f111f1, TABLES.round_constants[5]);
        assert_eq!(0x923f82a4, TABLES.round_constants[6]);
        assert_eq!(0xab1c5ed5, TABLES.round_constants[7]);
        assert_eq!(0xc67178f2, TABLES.round_constants[63]);
    }

    #[test]
    fn test_init_hash() {
        assert_eq!(0x6a09e667, TABLES.init_hash[0]);
        assert_eq!(0xbb67ae85, TABLES.init_hash[1]);
        assert_eq!(0x3c6ef372, TABLES.init_hash[2]);
        assert_eq!(0xa54ff53a, TABLES.init_hash[3]);
        assert_eq!(0x510e527f, TABLES.init_hash[4]);
        assert_eq!(0x9b05688c, TABLES.init_hash[5]);
        assert_eq!(0x1f83d9ab, TABLES.init_hash[6]);
        assert_eq!(0x5be0cd19, TABLES.init_hash[7]);
    }

    #[test]
    fn test_hash() {
        assert_eq!(
            vec![
                0xba, 0x78, 0x16, 0xbf, 0x8f, 0x01, 0xcf, 0xea, //
                0x41, 0x41, 0x40, 0xde, 0x5d, 0xae, 0x22, 0x23, //
                0xb0, 0x03, 0x61, 0xa3, 0x96, 0x17, 0x7a, 0x9c, //
                0xb4, 0x10, 0xff, 0x61, 0xf2, 0x00, 0x15, 0xad, //
            ],
            Sha256::hash(&[0x61, 0x62, 0x63])
        );
        assert_eq!(
            vec![
                0x24, 0x8d, 0x6a, 0x61, 0xd2, 0x06, 0x38, 0xb8, //
                0xe5, 0xc0, 0x26, 0x93, 0x0c, 0x3e, 0x60, 0x39, //
                0xa3, 0x3c, 0xe4, 0x59, 0x64, 0xff, 0x21, 0x67, //
                0xf6, 0xec, 0xed, 0xd4, 0x19, 0xdb, 0x06, 0xc1, //
            ],
            Sha256::hash(
                &[
                    0x61, 0x62, 0x63, 0x64, 0x62, 0x63, 0x64, 0x65, //
                    0x63, 0x64, 0x65, 0x66, 0x64, 0x65, 0x66, 0x67, //
                    0x65, 0x66, 0x67, 0x68, 0x66, 0x67, 0x68, 0x69, //
                    0x67, 0x68, 0x69, 0x6a, 0x68, 0x69, 0x6a, 0x6b, //
                    0x69, 0x6a, 0x6b, 0x6c, 0x6a, 0x6b, 0x6c, 0x6d, //
                    0x6b, 0x6c, 0x6d, 0x6e, 0x6c, 0x6d, 0x6e, 0x6f, //
                    0x6d, 0x6e, 0x6f, 0x70, 0x6e, 0x6f, 0x70, 0x71, //
                ][..]
            )
        );
        let v = 0x61;
        assert_eq!(
            vec![
                0xcd, 0xc7, 0x6e, 0x5c, 0x99, 0x14, 0xfb, 0x92, //
                0x81, 0xa1, 0xc7, 0xe2, 0x84, 0xd7, 0x3e, 0x67, //
                0xf1, 0x80, 0x9a, 0x48, 0xa4, 0x97, 0x20, 0x0e, //
                0x04, 0x6d, 0x39, 0xcc, 0xc7, 0x11, 0x2c, 0xd0, //
            ],
            Sha256::hash(iter::repeat(&v).take(1_000_000))
        );
    }
}
