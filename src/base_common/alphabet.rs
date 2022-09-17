use std::{error, fmt};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Error {
    DuplicateCharacter { character: char, first: usize, second: usize },
    NonAsciiCharacter { character: u8, index: usize },
    InvalidCharacter { character: char, index: usize },
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum DecodeError {
    NonAsciiCharacter { character: u8, index: usize },
    InvalidCharacter { character: char, index: usize },
}

impl error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateCharacter { character, first, second } => {
                write!(f, "Invalid character '{}' at indexes {} and {}", character, first, second)
            }
            Self::InvalidCharacter { character, index } => write!(f, "Invalid character '{}' at index {}", character, index),
            Self::NonAsciiCharacter { character, index } => write!(f, "Non-ascii character {:#02x} at index {}", character, index),
        }
    }
}

impl error::Error for DecodeError {}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidCharacter { character, index } => write!(f, "Invalid character '{}' at index {}", character, index),
            Self::NonAsciiCharacter { character, index } => write!(f, "Non-ascii character {:#02x} at index {}", character, index),
        }
    }
}

pub struct Alphabet<const N: usize> {
    encode: [u8; N],
    decode: [Option<u8>; 128],
}

impl<const N: usize> Alphabet<N> {
    pub fn encode(&self, value: usize) -> u8 {
        self.encode[value]
    }

    pub fn decode(&self, value: u8, index: usize) -> Result<u8, DecodeError> {
        if value >= 128 {
            return Err(DecodeError::NonAsciiCharacter { index, character: value });
        }
        match self.decode[value as usize] {
            Some(value) => Ok(value),
            None => Err(DecodeError::InvalidCharacter {
                character: value as char,
                index,
            }),
        }
    }

    pub const fn new(characters: &[u8; N]) -> Result<Self, Error> {
        let mut encode = [0u8; N];
        let mut decode: [Option<u8>; 128] = [None; 128];

        let mut index = 0;
        while index < encode.len() {
            let character = characters[index];
            if character >= 128 {
                return Err(Error::NonAsciiCharacter { index, character });
            }
            if let Some(v) = decode[characters[index] as usize] {
                return Err(Error::DuplicateCharacter {
                    character: character as char,
                    first: v as usize,
                    second: index,
                });
            }
            encode[index] = character;
            decode[character as usize] = Some(index as u8);
            index += 1;
        }

        Ok(Self { encode, decode })
    }

    pub const fn len(&self) -> usize {
        self.encode.len()
    }
}
