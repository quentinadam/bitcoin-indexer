use std::{error, fmt};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Error {
    BufferTooSmall,
    InvalidHexCharacter { character: char, index: usize },
    OddLength,
}

impl error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::BufferTooSmall => write!(f, "Output buffer too small"),
            Error::InvalidHexCharacter { character, index } => {
                write!(f, "Invalid character {:?} at position {}", character, index)
            }
            Error::OddLength => write!(f, "Odd number of digits"),
        }
    }
}

const fn value(character: u8, index: usize) -> Result<u8, Error> {
    match character {
        b'A'..=b'F' => Ok(character - b'A' + 10),
        b'a'..=b'f' => Ok(character - b'a' + 10),
        b'0'..=b'9' => Ok(character - b'0'),
        _ => Err(Error::InvalidHexCharacter {
            character: character as char,
            index,
        }),
    }
}

pub fn decode(input: impl AsRef<[u8]>) -> Result<Vec<u8>, Error> {
    let input = input.as_ref();
    let mut output = vec![0u8; input.len() / 2];
    let len = decode_into(input, &mut output)?;
    assert_eq!(len, output.len());
    Ok(output)
}

pub fn decode_into(input: impl AsRef<[u8]>, output: &mut impl AsMut<[u8]>) -> Result<usize, Error> {
    let input = input.as_ref();
    let output = output.as_mut();
    if input.len() % 2 != 0 {
        return Err(Error::OddLength);
    }
    let len = input.len() / 2;
    if output.len() < len {
        return Err(Error::BufferTooSmall);
    }
    for (i, pair) in input.chunks(2).enumerate() {
        output[i] = value(pair[0], 2 * i)? << 4 | value(pair[1], 2 * i + 1)?;
    }
    Ok(len)
}

const TABLE: &[u8; 16] = b"0123456789abcdef";

pub fn encode(input: impl AsRef<[u8]>) -> String {
    encode_iterator(input.as_ref().iter())
}

pub fn encode_iterator<'a>(input: impl ExactSizeIterator<Item = &'a u8>) -> String {
    let mut output = Vec::with_capacity(input.len() * 2);
    for byte in input {
        output.push(TABLE[(byte >> 4) as usize]);
        output.push(TABLE[(byte & 0x0F) as usize]);
    }
    unsafe { String::from_utf8_unchecked(output) }
}

#[cfg(test)]
mod tests {
    #[test]
    fn encode() {
        let output = super::encode(b"Hello world");
        assert_eq!(output, "48656c6c6f20776f726c64");
    }

    #[test]
    fn decode() {
        let output = super::decode("48656c6c6f20776f726c64");
        assert_eq!(output, Ok(vec![0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x77, 0x6f, 0x72, 0x6c, 0x64]));
    }

    #[test]
    fn decode_into() {
        let mut output = [0u8; 11];
        let len = super::decode_into("48656c6c6f20776f726c64", &mut output);
        assert_eq!(len, Ok(11));
        assert_eq!(output, [0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x77, 0x6f, 0x72, 0x6c, 0x64]);
    }
}
