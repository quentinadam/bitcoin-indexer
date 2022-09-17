use super::{compute_checksum, Alphabet, ALPHABET};
use crate::base58::{self, encode::Error};

pub struct Encoder<'a> {
    encoder: base58::Encoder<'a>,
}

impl<'a> Encoder<'a> {
    pub const fn new(alphabet: &'a Alphabet<58>) -> Self {
        Self {
            encoder: base58::Encoder::new(alphabet),
        }
    }

    fn extend_input(&self, input: impl AsRef<[u8]>) -> Vec<u8> {
        let mut input = input.as_ref().to_vec();
        let checksum = compute_checksum(&input);
        input.extend_from_slice(checksum.as_ref());
        input
    }

    pub fn encode(&self, input: impl AsRef<[u8]>) -> String {
        self.encoder.encode(&self.extend_input(input))
    }

    pub fn encode_into(&self, input: impl AsRef<[u8]>, output: &mut impl AsMut<[u8]>) -> Result<usize, Error> {
        self.encoder.encode_into(&self.extend_input(input), output)
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
