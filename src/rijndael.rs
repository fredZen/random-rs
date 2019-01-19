use crate::galois::Gf256;

mod key_expansion;
mod mix_columns;
mod sbox;

#[derive(Debug, PartialEq)]
pub struct Aes256 {
    state: [Gf256; 16],
}

fn add_round_key(Aes256 { state }: &mut Aes256, key: &[Gf256; 240], i: usize) {
    for (s, &k) in state.iter_mut().zip(key[i * 16..(i + 1) * 16].iter()) {
        *s += k;
    }
}

fn sub_bytes(Aes256 { state }: &mut Aes256) {
    let sbox = &*sbox::SBOX;

    for item in state.iter_mut() {
        *item = sbox.direct(*item);
    }
}

fn shift_rows(Aes256 { state }: &mut Aes256) {
    let mut t = state[1];

    state[1] = state[5];
    state[5] = state[9];
    state[9] = state[13];
    state[13] = t;

    t = state[2];
    state[2] = state[10];
    state[10] = t;
    t = state[6];
    state[6] = state[14];
    state[14] = t;

    t = state[3];
    state[3] = state[15];
    state[15] = state[11];
    state[11] = state[7];
    state[7] = t;
}

fn mix_columns(Aes256 { state }: &mut Aes256) {
    mix_columns::mix_column(&mut state[0..4]);
    mix_columns::mix_column(&mut state[4..8]);
    mix_columns::mix_column(&mut state[8..12]);
    mix_columns::mix_column(&mut state[12..16]);
}

pub fn encrypt_block(state: &mut Aes256, key: &[Gf256; 240]) {
    add_round_key(state, key, 0);

    for i in 1..14 {
        sub_bytes(state);
        shift_rows(state);
        mix_columns(state);
        add_round_key(state, key, i);
    }

    sub_bytes(state);
    shift_rows(state);
    add_round_key(state, key, 14);
}

#[cfg(test)]
mod tests {
    use super::*;
    use arraymap::ArrayMap;

    #[test]
    fn test_shift_rows() {
        let mut s = Aes256 {
            state: [
                0x00, 0x10, 0x20, 0x30, //
                0x01, 0x11, 0x21, 0x31, //
                0x02, 0x12, 0x22, 0x32, //
                0x03, 0x13, 0x23, 0x33, //
            ]
            .map(|&v| Gf256(v)),
        };
        shift_rows(&mut s);
        assert_eq!(
            s,
            Aes256 {
                state: [
                    0x00, 0x11, 0x22, 0x33, //
                    0x01, 0x12, 0x23, 0x30, //
                    0x02, 0x13, 0x20, 0x31, //
                    0x03, 0x10, 0x21, 0x32, //
                ]
                .map(|&v| Gf256(v))
            }
        );
    }

    #[test]
    fn test_encrypt_block() {
        let key = [
            0x60, 0x3d, 0xeb, 0x10, 0x15, 0xca, 0x71, 0xbe, //
            0x2b, 0x73, 0xae, 0xf0, 0x85, 0x7d, 0x77, 0x81, //
            0x1f, 0x35, 0x2c, 0x07, 0x3b, 0x61, 0x08, 0xd7, //
            0x2d, 0x98, 0x10, 0xa3, 0x09, 0x14, 0xdf, 0xf4, //
        ];
        let mut expanded_key = [Gf256(0); 240];

        for (&from, to) in key.iter().zip(expanded_key.iter_mut()) {
            *to = Gf256(from);
        }
        key_expansion::expand_key(&mut expanded_key);
        let mut block = Aes256 {
            state: [
                0xae, 0x2d, 0x8a, 0x57, 0x1e, 0x03, 0xac, 0x9c, //
                0x9e, 0xb7, 0x6f, 0xac, 0x45, 0xaf, 0x8e, 0x51, //
            ]
            .map(|&v| Gf256(v)),
        };
        encrypt_block(&mut block, &expanded_key);
        assert_eq!(
            block,
            Aes256 {
                state: [
                    0x59, 0x1c, 0xcb, 0x10, 0xd4, 0x10, 0xed, 0x26, //
                    0xdc, 0x5b, 0xa7, 0x4a, 0x31, 0x36, 0x28, 0x70, //
                ]
                .map(|&v| Gf256(v))
            }
        );
    }
}
