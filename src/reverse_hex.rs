use crate::hex::{self, Error};

pub fn decode(input: impl AsRef<[u8]>) -> Result<Vec<u8>, Error> {
    let mut output = hex::decode(input)?;
    output.reverse();
    Ok(output)
}

pub fn decode_into(input: impl AsRef<[u8]>, output: &mut impl AsMut<[u8]>) -> Result<usize, Error> {
    let len = hex::decode_into(input, output)?;
    output.as_mut()[0..len].reverse();
    Ok(len)
}

pub fn encode(input: impl AsRef<[u8]>) -> String {
    hex::encode_iterator(input.as_ref().iter().rev())
}

#[cfg(test)]
mod tests {
    #[test]
    fn encode() {
        let output = super::encode(b"Hello world");
        assert_eq!(output, "646c726f77206f6c6c6548");
    }

    #[test]
    fn decode() {
        let output = super::decode("48656c6c6f20776f726c64");
        assert_eq!(output, Ok(vec![0x64, 0x6c, 0x72, 0x6f, 0x77, 0x20, 0x6f, 0x6c, 0x6c, 0x65, 0x48]));
    }

    #[test]
    fn decode_into() {
        let mut output = [0u8; 11];
        let len = super::decode_into("48656c6c6f20776f726c64", &mut output);
        assert_eq!(len, Ok(11));
        assert_eq!(output, [0x64, 0x6c, 0x72, 0x6f, 0x77, 0x20, 0x6f, 0x6c, 0x6c, 0x65, 0x48]);
    }
}
