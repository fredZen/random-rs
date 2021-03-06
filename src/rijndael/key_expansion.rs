use crate::galois;
use crate::galois::Gf256;
use crate::rijndael::sbox;

fn rotate(r: &mut [Gf256; 4]) {
    let a = r[0];
    for c in 0..3 {
        r[c] = r[c + 1];
    }
    r[3] = a;
}

fn round_constant(i: usize) -> Gf256 {
    let mut a = Gf256(1);
    for _ in 1..i {
        a = galois::mul2(a);
    }

    a
}

fn apply_sbox(r: &mut [Gf256; 4]) {
    let sbox = &*sbox::SBOX;

    for item in r.iter_mut() {
        *item = sbox.direct(*item);
    }
}

fn schedule_core(r: &mut [Gf256; 4], i: usize) {
    rotate(r);
    apply_sbox(r);
    r[0] += round_constant(i);
}

#[allow(dead_code)]
pub fn expand_key(key: &mut [Gf256; 240]) {
    let mut t: [Gf256; 4] = Default::default();
    let mut i = 1;

    for c in (32..240).step_by(4) {
        t.copy_from_slice(&key[c - 4..c]);

        match c & 0x1f {
            0 => {
                schedule_core(&mut t, i);
                i += 1;
            }
            16 => {
                apply_sbox(&mut t);
            }
            _ => {}
        }

        for (a, &item) in t.iter().enumerate() {
            key[c + a] = key[c + a - 32] + item;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arraymap::ArrayMap;

    #[test]
    fn test_rotate() {
        let mut r = [0x1d, 0x2c, 0x3a, 0x4f].map(|&v| Gf256(v));
        rotate(&mut r);
        assert_eq!(r, [0x2c, 0x3a, 0x4f, 0x1d].map(|&v| Gf256(v)));
    }

    #[test]
    fn test_round_constant() {
        assert_eq!(Gf256(0x01), round_constant(1));
        assert_eq!(Gf256(0x02), round_constant(2));
        assert_eq!(Gf256(0x04), round_constant(3));
        assert_eq!(Gf256(0x08), round_constant(4));
        assert_eq!(Gf256(0x10), round_constant(5));
        assert_eq!(Gf256(0x20), round_constant(6));
        assert_eq!(Gf256(0x40), round_constant(7));
        assert_eq!(Gf256(0x80), round_constant(8));
        assert_eq!(Gf256(0x1b), round_constant(9));
        assert_eq!(Gf256(0x36), round_constant(10));
    }

    #[test]
    fn test_schedule_core() {
        let mut r = [Gf256(0); 4];
        schedule_core(&mut r, 1);
        assert_eq!(r, [0x62, 0x63, 0x63, 0x63].map(|&v| Gf256(v)));

        r = [0, 1, 2, 3].map(|&v| Gf256(v));
        schedule_core(&mut r, 2);
        assert_eq!(r, [0x7e, 0x77, 0x7b, 0x63].map(|&v| Gf256(v)));
    }

    #[test]
    fn test_expand_key_256() {
        let mut key = [Gf256(0); 240];
        expand_key(&mut key);
        assert_eq!(
            key.iter().map(|&Gf256(v)| v).collect::<Vec<u8>>(),
            vec![
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
                0x62, 0x63, 0x63, 0x63, 0x62, 0x63, 0x63, 0x63, //
                0x62, 0x63, 0x63, 0x63, 0x62, 0x63, 0x63, 0x63, //
                0xaa, 0xfb, 0xfb, 0xfb, 0xaa, 0xfb, 0xfb, 0xfb, //
                0xaa, 0xfb, 0xfb, 0xfb, 0xaa, 0xfb, 0xfb, 0xfb, //
                0x6f, 0x6c, 0x6c, 0xcf, 0x0d, 0x0f, 0x0f, 0xac, //
                0x6f, 0x6c, 0x6c, 0xcf, 0x0d, 0x0f, 0x0f, 0xac, //
                0x7d, 0x8d, 0x8d, 0x6a, 0xd7, 0x76, 0x76, 0x91, //
                0x7d, 0x8d, 0x8d, 0x6a, 0xd7, 0x76, 0x76, 0x91, //
                0x53, 0x54, 0xed, 0xc1, 0x5e, 0x5b, 0xe2, 0x6d, //
                0x31, 0x37, 0x8e, 0xa2, 0x3c, 0x38, 0x81, 0x0e, //
                0x96, 0x8a, 0x81, 0xc1, 0x41, 0xfc, 0xf7, 0x50, //
                0x3c, 0x71, 0x7a, 0x3a, 0xeb, 0x07, 0x0c, 0xab, //
                0x9e, 0xaa, 0x8f, 0x28, 0xc0, 0xf1, 0x6d, 0x45, //
                0xf1, 0xc6, 0xe3, 0xe7, 0xcd, 0xfe, 0x62, 0xe9, //
                0x2b, 0x31, 0x2b, 0xdf, 0x6a, 0xcd, 0xdc, 0x8f, //
                0x56, 0xbc, 0xa6, 0xb5, 0xbd, 0xbb, 0xaa, 0x1e, //
                0x64, 0x06, 0xfd, 0x52, 0xa4, 0xf7, 0x90, 0x17, //
                0x55, 0x31, 0x73, 0xf0, 0x98, 0xcf, 0x11, 0x19, //
                0x6d, 0xbb, 0xa9, 0x0b, 0x07, 0x76, 0x75, 0x84, //
                0x51, 0xca, 0xd3, 0x31, 0xec, 0x71, 0x79, 0x2f, //
                0xe7, 0xb0, 0xe8, 0x9c, 0x43, 0x47, 0x78, 0x8b, //
                0x16, 0x76, 0x0b, 0x7b, 0x8e, 0xb9, 0x1a, 0x62, //
                0x74, 0xed, 0x0b, 0xa1, 0x73, 0x9b, 0x7e, 0x25, //
                0x22, 0x51, 0xad, 0x14, 0xce, 0x20, 0xd4, 0x3b, //
                0x10, 0xf8, 0x0a, 0x17, 0x53, 0xbf, 0x72, 0x9c, //
                0x45, 0xc9, 0x79, 0xe7, 0xcb, 0x70, 0x63, 0x85, //
            ]
        )
    }
}
