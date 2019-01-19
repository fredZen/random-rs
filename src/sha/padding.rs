use std::iter;
use std::iter::Extend;

pub fn pad_448(message: &mut Vec<u8>) {
    let l = message.len();
    message.push(0x80);
    message.extend(iter::repeat(0).take((63 + (-72 - l as i8) % 64) as usize));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pad_448_with_single_byte() {
        let mut message = vec![0xae];
        pad_448(&mut message);
        assert_eq!(56, message.len());

        let mut expected = vec![0xae, 0x80];
        expected.extend(iter::repeat(0).take(54));
        assert_eq!(message, expected);
    }

    #[test]
    fn test_pad_448_just_fits_448() {
        let mut message = vec![0xae; 55];
        pad_448(&mut message);
        assert_eq!(56, message.len());

        let mut expected = vec![0xae; 55];
        expected.push(0x80);
        assert_eq!(message, expected);
    }

    #[test]
    fn test_pad_448_just_over_448() {
        let mut message = vec![0xae; 56];
        pad_448(&mut message);
        assert_eq!(120, message.len());

        let mut expected = vec![0xae; 56];
        expected.push(0x80);
        expected.extend(iter::repeat(0).take(63));
        assert_eq!(message, expected);
    }
}
