use crate::base_common::Alphabet;
use std::{error, fmt};

use super::ALPHABET;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Error {
    /// The output buffer was too small to contain the entire input.
    BufferTooSmall,
}

impl error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::BufferTooSmall => write!(f, "Output buffer too small"),
        }
    }
}

pub struct Encoder<'a> {
    alphabet: &'a Alphabet<58>,
}

impl<'a> Encoder<'a> {
    pub const fn new(alphabet: &'a Alphabet<58>) -> Self {
        Self { alphabet }
    }

    pub fn encode_into(&self, input: impl AsRef<[u8]>, output: &mut impl AsMut<[u8]>) -> Result<usize, Error> {
        let input = input.as_ref();
        let output = output.as_mut();
        let mut index = 0;
        for &value in input {
            let mut carry = value as usize;
            for value in &mut output[..index] {
                carry += (*value as usize) << 8;
                *value = (carry % 58) as u8;
                carry /= 58;
            }
            while carry > 0 {
                *output.get_mut(index).ok_or(Error::BufferTooSmall)? = (carry % 58) as u8;
                index += 1;
                carry /= 58;
            }
        }
        for _ in input.iter().take_while(|&&value| value == 0) {
            *output.get_mut(index).ok_or(Error::BufferTooSmall)? = 0;
            index += 1;
        }
        for value in &mut output[..] {
            *value = self.alphabet.encode(*value as usize);
        }
        output[0..index].reverse();
        Ok(index)
    }

    pub fn encode(&self, input: impl AsRef<[u8]>) -> String {
        let mut output = vec![0u8; (input.as_ref().len() * 8) / 5 + 1];
        let len = self.encode_into(input, &mut output).unwrap();
        output.truncate(len);
        unsafe { String::from_utf8_unchecked(output) }
    }

    pub fn default() -> &'static Self {
        &ENCODER
    }
}

const ENCODER: Encoder = Encoder::new(&ALPHABET);

pub fn encode(input: impl AsRef<[u8]>) -> String {
    Encoder::default().encode(input)
}

pub fn encode_into(input: impl AsRef<[u8]>, output: &mut impl AsMut<[u8]>) -> Result<usize, Error> {
    Encoder::default().encode_into(input, output)
}

#[cfg(test)]
mod tests {
    #[test]
    fn encode() {
        assert_eq!(super::encode([]), "");
        assert_eq!(super::encode("a"), "2g");
        assert_eq!(super::encode("bbb"), "a3gV");
        assert_eq!(super::encode("ccc"), "aPEr");
        assert_eq!(super::encode("simply a long string"), "2cFupjhnEsSn59qHXstmK2ffpLv2");
        assert_eq!(super::encode("simply a long string"), "2cFupjhnEsSn59qHXstmK2ffpLv2");
        assert_eq!(
            super::encode([
                0x00, 0xeb, 0x15, 0x23, 0x1d, 0xfc, 0xeb, 0x60, 0x92, 0x58, 0x86, 0xb6, 0x7d, 0x06, 0x52, 0x99, 0x92, 0x59, 0x15, 0xae,
                0xb1, 0x72, 0xc0, 0x66, 0x47,
            ]),
            "1NS17iag9jJgTHD1VXjvLCEnZuQ3rJDE9L"
        );
        assert_eq!(super::encode([0x51, 0x6b, 0x6f, 0xcd, 0x0f]), "ABnLTmg");
        assert_eq!(
            super::encode([0xbf, 0x4f, 0x89, 0x00, 0x1e, 0x67, 0x02, 0x74, 0xdd]),
            "3SEo3LWLoPntC"
        );
        assert_eq!(super::encode([0x57, 0x2e, 0x47, 0x94]), "3EFU7m");

        assert_eq!(
            super::encode([0xec, 0xac, 0x89, 0xca, 0xd9, 0x39, 0x23, 0xc0, 0x23, 0x21]),
            "EJDM8drfXA6uyA"
        );
        assert_eq!(super::encode([0x10, 0xc8, 0x51, 0x1e,]), "Rt5zm");
        assert_eq!(
            super::encode([0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,]),
            "1111111111"
        );
    }
}
