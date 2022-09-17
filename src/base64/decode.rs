use super::{Alphabet, ALPHABET};
pub use crate::base_binary::decode::Error;

pub struct Decoder<'a> {
    decoder: crate::base_binary::Decoder<'a, 64>,
}

impl<'a> Decoder<'a> {
    pub const fn new(alphabet: &'a Alphabet<64>) -> Self {
        Self {
            decoder: crate::base_binary::Decoder::new(alphabet, 6),
        }
    }

    pub fn decode_into(&self, input: impl AsRef<[u8]>, output: &mut impl AsMut<[u8]>) -> Result<usize, Error> {
        self.decoder.decode_into(input, output)
    }

    pub fn decode(&self, input: impl AsRef<[u8]>) -> Result<Vec<u8>, Error> {
        self.decoder.decode(input)
    }

    pub fn default() -> &'static Self {
        &DECODER
    }
}

const DECODER: Decoder = Decoder::new(&ALPHABET);

pub fn decode_into(input: impl AsRef<[u8]>, output: &mut impl AsMut<[u8]>) -> Result<usize, Error> {
    Decoder::default().decode_into(input, output)
}

pub fn decode(input: impl AsRef<[u8]>) -> Result<Vec<u8>, Error> {
    Decoder::default().decode(input)
}

#[cfg(test)]
mod tests {
    #[test]
    fn decode() {
        assert_eq!(super::decode("FPucA9l+"), Ok(vec![0x14, 0xfb, 0x9c, 0x03, 0xd9, 0x7e]));
        assert_eq!(super::decode("FPucA9k="), Ok(vec![0x14, 0xfb, 0x9c, 0x03, 0xd9]));
        assert_eq!(super::decode("FPucAw=="), Ok(vec![0x14, 0xfb, 0x9c, 0x03]));
        assert_eq!(super::decode(""), Ok(b"".to_vec()));
        assert_eq!(super::decode("Zg=="), Ok(b"f".to_vec()));
        assert_eq!(super::decode("Zm8="), Ok(b"fo".to_vec()));
        assert_eq!(super::decode("Zm9v"), Ok(b"foo".to_vec()));
        assert_eq!(super::decode("Zm9vYg=="), Ok(b"foob".to_vec()));
        assert_eq!(super::decode("Zm9vYmE="), Ok(b"fooba".to_vec()));
        assert_eq!(super::decode("Zm9vYmFy"), Ok(b"foobar".to_vec()));
    }
}
