use super::Alphabet;
use std::{error, fmt};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Error {
    BufferTooSmall,
}

impl error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::BufferTooSmall => write!(f, "Output buffer too small"),
        }
    }
}

fn div_ceil(lhs: usize, rhs: usize) -> usize {
    ((-(lhs as isize)).div_euclid(-(rhs as isize))) as usize
}

pub struct Encoder<'a, const N: usize> {
    alphabet: &'a Alphabet<N>,
    bits: usize,
}

impl<'a, const N: usize> Encoder<'a, N> {
    pub const fn new(alphabet: &'a Alphabet<N>, bits: usize) -> Self {
        assert!(alphabet.len() == (1 << bits));
        Self { alphabet, bits }
    }

    pub fn encode_into(&self, input: impl AsRef<[u8]>, output: &mut impl AsMut<[u8]>) -> Result<usize, Error> {
        let output = output.as_mut();
        let mut accumulator: usize = 0;
        let mut bits: usize = 0;
        let mut index = 0;
        for &value in input.as_ref() {
            accumulator = (accumulator << 8) | (value as usize);
            bits += 8;
            while bits >= self.bits {
                bits -= self.bits;
                *output.get_mut(index).ok_or(Error::BufferTooSmall)? = self.alphabet.encode(accumulator >> bits);
                index += 1;
                accumulator = accumulator & ((1 << bits) - 1);
            }
        }
        if bits > 0 {
            *output.get_mut(index).ok_or(Error::BufferTooSmall)? = self.alphabet.encode(accumulator << (self.bits - bits));
            index += 1;
        }
        while (index * self.bits) % 8 != 0 {
            *output.get_mut(index).ok_or(Error::BufferTooSmall)? = 0x3d;
            index += 1;
        }
        Ok(index)
    }

    pub fn encode(&self, input: impl AsRef<[u8]>) -> String {
        let mut output = vec![0u8; div_ceil(input.as_ref().len(), self.bits) * 8];
        let len = self.encode_into(input, &mut output).unwrap();
        output.truncate(len);
        unsafe { String::from_utf8_unchecked(output) }
    }
}
