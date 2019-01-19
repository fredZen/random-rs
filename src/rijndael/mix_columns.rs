use crate::galois;
use crate::galois::Gf256;

pub fn mix_column(r: &mut [Gf256]) {
    let mut a: [Gf256; 4] = Default::default();
    a.copy_from_slice(r);
    let mut b: [Gf256; 4] = Default::default();

    for (c, item) in a.iter().enumerate() {
        b[c] = galois::mul2(*item);
    }

    r[0] = b[0] + a[3] + a[2] + b[1] + a[1];
    r[1] = b[1] + a[0] + a[3] + b[2] + a[2];
    r[2] = b[2] + a[1] + a[0] + b[3] + a[3];
    r[3] = b[3] + a[2] + a[1] + b[0] + a[0];
}

#[allow(dead_code)]
pub fn inv_mix_column(r: &mut [Gf256; 4]) {
    let mut a: [Gf256; 4] = Default::default();
    a.copy_from_slice(r);

    r[0] = a[0] * Gf256(14) + a[3] * Gf256(9) + a[2] * Gf256(13) + a[1] * Gf256(11);
    r[1] = a[1] * Gf256(14) + a[0] * Gf256(9) + a[3] * Gf256(13) + a[2] * Gf256(11);
    r[2] = a[2] * Gf256(14) + a[1] * Gf256(9) + a[0] * Gf256(13) + a[3] * Gf256(11);
    r[3] = a[3] * Gf256(14) + a[2] * Gf256(9) + a[1] * Gf256(13) + a[0] * Gf256(11);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mix_column() {
        let mut r = [Gf256(1), Gf256(1), Gf256(1), Gf256(1)];
        mix_column(&mut r);
        assert_eq!(r, [Gf256(1), Gf256(1), Gf256(1), Gf256(1)]);

        r = [Gf256(0xdb), Gf256(0x13), Gf256(0x53), Gf256(0x45)];
        mix_column(&mut r);
        assert_eq!(r, [Gf256(0x8e), Gf256(0x4d), Gf256(0xa1), Gf256(0xbc)]);

        r = [Gf256(0xf2), Gf256(0x0a), Gf256(0x22), Gf256(0x5c)];
        mix_column(&mut r);
        assert_eq!(r, [Gf256(0x9f), Gf256(0xdc), Gf256(0x58), Gf256(0x9d)]);
    }

    #[test]
    fn test_inv_mix_column() {
        let mut r = [Gf256(1), Gf256(1), Gf256(1), Gf256(1)];
        inv_mix_column(&mut r);
        assert_eq!(r, [Gf256(1), Gf256(1), Gf256(1), Gf256(1)]);

        r = [Gf256(0x8e), Gf256(0x4d), Gf256(0xa1), Gf256(0xbc)];
        inv_mix_column(&mut r);
        assert_eq!(r, [Gf256(0xdb), Gf256(0x13), Gf256(0x53), Gf256(0x45)]);

        r = [Gf256(0x9f), Gf256(0xdc), Gf256(0x58), Gf256(0x9d)];
        inv_mix_column(&mut r);
        assert_eq!(r, [Gf256(0xf2), Gf256(0x0a), Gf256(0x22), Gf256(0x5c)]);
    }
}
