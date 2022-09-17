use super::{compute_checksum, Alphabet, ALPHABET};
use crate::base58::{self};
use std::convert::TryInto;
use std::{error, fmt};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Error {
    BufferTooSmall,
    InvalidCharacter { character: char, index: usize },
    NonAsciiCharacter { index: usize, character: u8 },
    InvalidChecksum { checksum: [u8; 4], expected_checksum: [u8; 4] },
    NoChecksum,
}

impl From<base58::decode::Error> for Error {
    fn from(error: base58::decode::Error) -> Self {
        match error {
            base58::decode::Error::BufferTooSmall => Error::BufferTooSmall,
            base58::decode::Error::InvalidCharacter { character, index } => Error::InvalidCharacter { character, index },
            base58::decode::Error::NonAsciiCharacter { character, index } => Error::NonAsciiCharacter { character, index },
        }
    }
}

impl error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::BufferTooSmall => write!(f, "Output buffer too small"),
            Error::InvalidCharacter { character, index } => write!(f, "Invalid character '{}' at index {}", character, index),
            Error::NonAsciiCharacter { character, index } => write!(f, "Non-ascii character {:#02x} at index {}", character, index),
            Error::InvalidChecksum {
                checksum,
                expected_checksum,
            } => write!(
                f,
                "Invalid checksum '{}' ({} expected)",
                HexSlice::new(&checksum),
                HexSlice::new(&expected_checksum)
            ),
            Error::NoChecksum => write!(f, "Missing checksum"),
        }
    }
}

struct HexSlice<'a> {
    buffer: &'a [u8],
}

impl<'a> HexSlice<'a> {
    fn new(buffer: &'a impl AsRef<[u8]>) -> HexSlice<'a> {
        HexSlice { buffer: buffer.as_ref() }
    }
}

impl fmt::Display for HexSlice<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.buffer {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

pub struct Decoder<'a> {
    decoder: base58::Decoder<'a>,
}

impl<'a> Decoder<'a> {
    pub const fn new(alphabet: &'a Alphabet<58>) -> Self {
        Self {
            decoder: base58::Decoder::new(alphabet),
        }
    }

    pub fn decode_into(&self, input: impl AsRef<[u8]>, output: &mut impl AsMut<[u8]>) -> Result<usize, Error> {
        let len = self.decoder.decode_into(input, output)?;
        verify_checksum(&output.as_mut()[..len])?;
        Ok(len - 4)
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

fn verify_checksum(buffer: &[u8]) -> Result<(), Error> {
    if buffer.len() < 4 {
        return Err(Error::NoChecksum);
    }
    let checksum = &buffer[buffer.len() - 4..];
    let expected_checksum = compute_checksum(&buffer[..buffer.len() - 4]);
    if checksum != &expected_checksum[..] {
        return Err(Error::InvalidChecksum {
            checksum: checksum.try_into().unwrap(),
            expected_checksum: expected_checksum[..].try_into().unwrap(),
        });
    }
    Ok(())
}

pub fn decode(input: impl AsRef<[u8]>) -> Result<Vec<u8>, Error> {
    Decoder::default().decode(input)
}

pub fn decode_into(input: impl AsRef<[u8]>, output: &mut impl AsMut<[u8]>) -> Result<usize, Error> {
    Decoder::default().decode_into(input, output)
}
