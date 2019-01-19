use std::ops::{Add, Sub};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct U255(pub u8);

impl From<U255> for usize {
    fn from(n: U255) -> Self {
        n.0 as usize
    }
}

impl Add for U255 {
    type Output = U255;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn add(self, other: U255) -> U255 {
        let s = u16::from(self.0) + u16::from(other.0);
        U255((s % 255) as u8)
    }
}

impl Sub for U255 {
    type Output = U255;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn sub(self, other: U255) -> U255 {
        let d = 255 + u16::from(self.0) - u16::from(other.0);
        U255((d % 255) as u8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(U255(123), U255(88) + U255(35));
        assert_eq!(U255(0), U255(200) + U255(55));
        assert_eq!(U255(45), U255(200) + U255(100));
        assert_eq!(U255(0), U255(255) + U255(255));
    }

    #[test]
    fn test_sub() {
        assert_eq!(U255(27), U255(42) - U255(15));
        assert_eq!(U255(10), U255(5) - U255(250));
        assert_eq!(U255(0), U255(0) - U255(255));
    }
}
