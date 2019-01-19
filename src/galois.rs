use lazy_static::lazy_static;

use crate::mod255::U255;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};
use volatile::Volatile;

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Gf256(pub u8);

pub struct Log {
    log: [U255; 256],
    exp: [Gf256; 256],
}

impl From<Gf256> for u8 {
    fn from(Gf256(n): Gf256) -> Self {
        n
    }
}

impl From<Gf256> for usize {
    fn from(Gf256(n): Gf256) -> Self {
        n as usize
    }
}

pub fn mul2(Gf256(n): Gf256) -> Gf256 {
    let carry = if (n & 0x80) != 0 { 0x1b } else { 0 };
    Gf256(n << 1 ^ carry)
}

pub fn mul3(n: Gf256) -> Gf256 {
    n + mul2(n)
}

lazy_static! {
    static ref LOG3: Log = {
        let mut log = [U255(0); 256];
        let mut exp = [Gf256(0); 256];
        let mut acc = Gf256(1);

        for (pow, item) in exp.iter_mut().enumerate() {
            *item = acc;
            log[usize::from(acc)] = U255(pow as u8);
            acc = mul3(acc)
        }

        Log { log, exp }
    };
}

pub fn exp(log: &Log, U255(n): U255) -> Gf256 {
    log.exp[n as usize]
}

pub fn exp3(n: U255) -> Gf256 {
    exp(&LOG3, n)
}

pub fn log(log: &Log, Gf256(n): Gf256) -> U255 {
    log.log[n as usize]
}

pub fn log3(n: Gf256) -> U255 {
    log(&LOG3, n)
}

impl Add for Gf256 {
    type Output = Gf256;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn add(self, Gf256(n): Gf256) -> Gf256 {
        Gf256(self.0 ^ n)
    }
}

impl AddAssign for Gf256 {
    fn add_assign(&mut self, Gf256(n): Gf256) {
        self.0 ^= n;
    }
}

impl Sub for Gf256 {
    type Output = Gf256;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn sub(self, Gf256(n): Gf256) -> Gf256 {
        Gf256(self.0 ^ n)
    }
}

impl SubAssign for Gf256 {
    fn sub_assign(&mut self, Gf256(n): Gf256) {
        self.0 ^= n;
    }
}

impl Mul for Gf256 {
    type Output = Gf256;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn mul(self, other: Gf256) -> Gf256 {
        let log3 = &LOG3;
        let mut s = Volatile::new(exp(log3, log(log3, self) + log(log3, other)));

        /* Now, we have some fancy code that returns 0 if either
        a or b are zero; we write the code this way so that the
        code will (hopefully) run at a constant speed in order to
        minimize the risk of timing attacks */

        let mut q = Volatile::new(s.read());
        let z = Volatile::new(Gf256(0));

        if self == Gf256(0) {
            s.write(z.read());
        } else {
            s.write(q.read());
        }

        if other == Gf256(0) {
            s.write(z.read());
        } else {
            q.write(z.read());
        }

        s.read()
    }
}

impl MulAssign for Gf256 {
    fn mul_assign(&mut self, other: Gf256) {
        let Gf256(u) = *self * other;
        self.0 = u;
    }
}

impl Div for Gf256 {
    type Output = Gf256;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn div(self, other: Gf256) -> Gf256 {
        assert_ne!(Gf256(0), other, "division by zero");
        let log3 = &LOG3;
        exp(log3, log(log3, self) - log(log3, other))
    }
}

impl DivAssign for Gf256 {
    fn div_assign(&mut self, other: Gf256) {
        let Gf256(u) = *self / other;
        self.0 = u;
    }
}

pub fn inv(n: Gf256) -> Gf256 {
    Gf256(1) / n
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        let mut a = Gf256(0b11100);
        let b = Gf256(0b110);

        assert_eq!(Gf256(0b11010), a + b);

        a += b;
        assert_eq!(Gf256(0b11010), a);
    }

    #[test]
    fn test_sub() {
        let mut a = Gf256(153);
        let b = Gf256(153);

        assert_eq!(Gf256(0), a - b);

        a -= b;
        assert_eq!(Gf256(0), a);
    }

    #[test]
    fn test_exp3() {
        assert_eq!(Gf256(0x01), exp3(U255(0x00)));
        assert_eq!(Gf256(0x03), exp3(U255(0x01)));
        assert_eq!(Gf256(0xc2), exp3(U255(0x87)));
        assert_eq!(Gf256(0x01), exp3(U255(0xff)));
    }

    #[test]
    fn test_log3() {
        assert_eq!(U255(0x00), log3(Gf256(0x00)));
        assert_eq!(U255(0xff), log3(Gf256(0x01)));
        assert_eq!(U255(0x19), log3(Gf256(0x02)));
        assert_eq!(U255(0x59), log3(Gf256(0x98)));
    }

    #[test]
    fn test_mul() {
        assert_eq!(Gf256(9), Gf256(3) * Gf256(7));
        assert_eq!(Gf256(1), Gf256(0x53) * Gf256(0xca));
        assert_eq!(Gf256(2), Gf256(1) * Gf256(2));
        assert_eq!(Gf256(2), Gf256(2) * Gf256(1));

        let mut a = Gf256(3);
        a *= Gf256(7);
        assert_eq!(Gf256(9), a);
    }

    #[test]
    fn test_div() {
        assert_eq!(Gf256(7), Gf256(9) / Gf256(3));

        let mut a = Gf256(9);
        a /= Gf256(3);
        assert_eq!(Gf256(7), a);
    }

    #[test]
    fn test_inv() {
        assert_eq!(Gf256(0x4e), inv(Gf256(0xe9)));
    }
}
