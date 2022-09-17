use crate::base58_check;
use crate::{Error, TryInto};
use std::{error, fmt};

#[derive(Eq, PartialEq, Hash, Debug, Clone)]
pub enum Address {
    P2PKH([u8; 20]),
    P2SH([u8; 20]),
}

impl Address {
    #[inline(always)]
    pub fn from_script(script: &[u8]) -> Result<Address, Error> {
        if script.len() == 25 && script[0] == 0x76 && script[1] == 0xA9 && script[2] == 0x14 && script[23] == 0x88 && script[24] == 0xAC {
            return Ok(Address::P2PKH(script[3..23].try_into().unwrap()));
        }
        if script.len() == 23 && script[0] == 0xa9 && script[1] == 0x14 && script[22] == 0x87 {
            return Ok(Address::P2SH(script[2..22].try_into().unwrap()));
        }
        Err(Error::new("Invalid address script"))
    }

    #[inline(always)]
    pub fn from_slice(buffer: impl AsRef<[u8]>) -> Result<Address, Box<dyn error::Error>> {
        let buffer = buffer.as_ref();
        if buffer.len() == 21 {
            match buffer[0] {
                0 => Ok(Address::P2PKH(buffer[1..].try_into().unwrap())),
                5 => Ok(Address::P2SH(buffer[1..].try_into().unwrap())),
                _ => Err(Error::new("Invalid address buffer").into()),
            }
        } else {
            Err(Error::new("Address buffer must be 21 bytes long").into())
        }
    }

    #[inline(always)]
    pub fn from_string(string: &str) -> Result<Address, Box<dyn error::Error>> {
        let mut buffer = [0u8; 25];
        let len = base58_check::decode_into(string, &mut buffer)?;
        Self::from_slice(&buffer[0..len])
    }

    #[inline(always)]
    pub fn to_vec(&self) -> Vec<u8> {
        let (version, hash) = match self {
            Address::P2PKH(hash) => (0, hash),
            Address::P2SH(hash) => (5, hash),
        };
        let mut vec = vec![version];
        vec.extend_from_slice(hash);
        vec
    }

    #[inline(always)]
    pub fn to_string(&self) -> String {
        base58_check::encode(self.to_vec())
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}
