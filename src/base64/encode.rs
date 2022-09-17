use super::{Alphabet, ALPHABET};
pub use crate::base_binary::encode::Error;

pub struct Encoder<'a> {
    encoder: crate::base_binary::Encoder<'a, 64>,
}

impl<'a> Encoder<'a> {
    pub const fn new(alphabet: &'a Alphabet<64>) -> Self {
        Self {
            encoder: crate::base_binary::Encoder::new(alphabet, 6),
        }
    }

    pub fn encode_into(&self, input: impl AsRef<[u8]>, output: &mut impl AsMut<[u8]>) -> Result<usize, Error> {
        self.encoder.encode_into(input, output)
    }

    pub fn encode(&self, input: impl AsRef<[u8]>) -> String {
        self.encoder.encode(input)
    }

    pub fn default() -> &'static Self {
        &ENCODER
    }
}

const ENCODER: Encoder = Encoder::new(&ALPHABET);

pub fn encode_into(input: impl AsRef<[u8]>, output: &mut impl AsMut<[u8]>) -> Result<usize, Error> {
    Encoder::default().encode_into(input, output)
}

pub fn encode(input: impl AsRef<[u8]>) -> String {
    Encoder::default().encode(input)
}

#[cfg(test)]
mod tests {
    #[test]
    fn encode() {
        assert_eq!(super::encode([0x14, 0xfb, 0x9c, 0x03, 0xd9, 0x7e]), "FPucA9l+");
        assert_eq!(super::encode([0x14, 0xfb, 0x9c, 0x03, 0xd9]), "FPucA9k=");
        assert_eq!(super::encode([0x14, 0xfb, 0x9c, 0x03]), "FPucAw==");
        assert_eq!(super::encode(b""), "");
        assert_eq!(super::encode(b"f"), "Zg==");
        assert_eq!(super::encode(b"fo"), "Zm8=");
        assert_eq!(super::encode(b"foo"), "Zm9v");
        assert_eq!(super::encode(b"foob"), "Zm9vYg==");
        assert_eq!(super::encode(b"fooba"), "Zm9vYmE=");
        assert_eq!(super::encode(b"foobar"), "Zm9vYmFy");
    }
}
