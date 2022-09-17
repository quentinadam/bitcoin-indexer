use super::{Alphabet, ALPHABET};
use crate::base_common::alphabet;
use std::{error, fmt};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Error {
    BufferTooSmall,
    InvalidCharacter { character: char, index: usize },
    NonAsciiCharacter { index: usize, character: u8 },
}

impl error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::BufferTooSmall => write!(f, "Output buffer too small"),
            Error::InvalidCharacter { character, index } => write!(f, "Invalid character '{}' at index {}", character, index),
            Error::NonAsciiCharacter { character, index } => write!(f, "Non-ascii character {:#02x} at index {}", character, index),
        }
    }
}

impl From<alphabet::DecodeError> for Error {
    fn from(error: alphabet::DecodeError) -> Self {
        match error {
            alphabet::DecodeError::InvalidCharacter { character, index } => Error::InvalidCharacter { character, index },
            alphabet::DecodeError::NonAsciiCharacter { character, index } => Error::NonAsciiCharacter { character, index },
        }
    }
}

pub struct Decoder<'a> {
    alphabet: &'a Alphabet<58>,
}

impl<'a> Decoder<'a> {
    pub const fn new(alphabet: &'a Alphabet<58>) -> Self {
        Self { alphabet }
    }

    pub fn decode_into(&self, input: impl AsRef<[u8]>, output: &mut impl AsMut<[u8]>) -> Result<usize, Error> {
        let input = input.as_ref();
        let output = output.as_mut();

        let mut output_index = 0;

        for (input_index, &value) in input.iter().enumerate() {
            let mut carry = self.alphabet.decode(value, input_index)? as usize;

            for value in &mut output[..output_index] {
                carry += (*value as usize) * 58;
                *value = (carry & 0xFF) as u8;
                carry >>= 8;
            }

            while carry > 0 {
                let value = output.get_mut(output_index).ok_or(Error::BufferTooSmall)?;
                *value = (carry & 0xFF) as u8;
                output_index += 1;
                carry >>= 8;
            }
        }

        let zero = self.alphabet.encode(0);

        for _ in input.iter().take_while(|&&value| value == zero) {
            let value = output.get_mut(output_index).ok_or(Error::BufferTooSmall)?;
            *value = 0;
            output_index += 1;
        }
        output[0..output_index].reverse();
        Ok(output_index)
    }

    pub fn decode(&self, input: impl AsRef<[u8]>) -> Result<Vec<u8>, Error> {
        let mut output = vec![0u8; input.as_ref().len()];
        let len = self.decode_into(input, &mut output)?;
        output.truncate(len);
        Ok(output)
    }

    pub fn default() -> &'static Self {
        &DECODER
    }
}

const DECODER: Decoder = Decoder::new(&ALPHABET);

pub fn decode(input: impl AsRef<[u8]>) -> Result<Vec<u8>, Error> {
    Decoder::default().decode(input)
}

pub fn decode_into(input: impl AsRef<[u8]>, output: &mut impl AsMut<[u8]>) -> Result<usize, Error> {
    Decoder::default().decode_into(input, output)
}

#[cfg(test)]
mod tests {
    #[test]
    fn decode() {
        assert_eq!(super::decode(""), Ok(vec![]));
        assert_eq!(super::decode("2g"), Ok(b"a".to_vec()));
        assert_eq!(super::decode("a3gV"), Ok(b"bbb".to_vec()));
        assert_eq!(super::decode("aPEr"), Ok(b"ccc".to_vec()));
        assert_eq!(super::decode("2cFupjhnEsSn59qHXstmK2ffpLv2"), Ok(b"simply a long string".to_vec()));
        assert_eq!(
            super::decode("1NS17iag9jJgTHD1VXjvLCEnZuQ3rJDE9L"),
            Ok(vec![
                0x00, 0xeb, 0x15, 0x23, 0x1d, 0xfc, 0xeb, 0x60, 0x92, 0x58, 0x86, 0xb6, 0x7d, 0x06, 0x52, 0x99, 0x92, 0x59, 0x15, 0xae,
                0xb1, 0x72, 0xc0, 0x66, 0x47,
            ])
        );
        assert_eq!(super::decode("ABnLTmg"), Ok(vec![0x51, 0x6b, 0x6f, 0xcd, 0x0f]));
        assert_eq!(
            super::decode("3SEo3LWLoPntC"),
            Ok(vec![0xbf, 0x4f, 0x89, 0x00, 0x1e, 0x67, 0x02, 0x74, 0xdd]),
        );
        assert_eq!(super::decode("3EFU7m"), Ok(vec![0x57, 0x2e, 0x47, 0x94]));

        assert_eq!(
            super::decode("EJDM8drfXA6uyA"),
            Ok(vec![0xec, 0xac, 0x89, 0xca, 0xd9, 0x39, 0x23, 0xc0, 0x23, 0x21]),
        );
        assert_eq!(super::decode("Rt5zm"), Ok(vec![0x10, 0xc8, 0x51, 0x1e,]));
        assert_eq!(
            super::decode("1111111111"),
            Ok(vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,])
        );
    }
}
