use super::Alphabet;
use crate::base_common::alphabet;
use std::{error, fmt};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Error {
    BufferTooSmall,
    NonAsciiCharacter { character: u8, index: usize },
    InvalidCharacter { character: char, index: usize },
}

impl error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::BufferTooSmall => write!(f, "Output buffer too small"),
            Self::InvalidCharacter { character, index } => write!(f, "Invalid character '{}' at index {}", character, index),
            Self::NonAsciiCharacter { character, index } => write!(f, "Non-ascii character {:#02x} at index {}", character, index),
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

pub struct Decoder<'a, const N: usize> {
    alphabet: &'a Alphabet<N>,
    bits: usize,
}

impl<'a, const N: usize> Decoder<'a, N> {
    pub const fn new(alphabet: &'a Alphabet<N>, bits: usize) -> Self {
        assert!(alphabet.len() == (1 << bits));
        Self { alphabet, bits }
    }

    pub fn decode_into(&self, input: impl AsRef<[u8]>, output: &mut impl AsMut<[u8]>) -> Result<usize, Error> {
        let output = output.as_mut();
        let mut accumulator: usize = 0;
        let mut bits: usize = 0;
        let mut output_index = 0;
        for (input_index, &value) in input.as_ref().iter().enumerate() {
            if value == 0x3d {
                continue;
            }
            let byte = self.alphabet.decode(value, input_index)?;
            accumulator = (accumulator << self.bits) | (byte as usize);
            bits += self.bits;
            while bits >= 8 {
                bits -= 8;
                *output.get_mut(output_index).ok_or(Error::BufferTooSmall)? = (accumulator >> bits) as u8;
                output_index += 1;
                accumulator = accumulator & ((1 << bits) - 1);
            }
        }
        assert!(accumulator == 0);
        Ok(output_index)
    }

    pub fn decode(&self, input: impl AsRef<[u8]>) -> Result<Vec<u8>, Error> {
        let mut output = vec![0u8; (input.as_ref().len() * self.bits) / 8];
        let len = self.decode_into(input, &mut output)?;
        output.truncate(len);
        Ok(output)
    }
}
