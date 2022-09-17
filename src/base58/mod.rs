pub mod decode;
pub mod encode;
pub use crate::base_common::Alphabet;

pub use decode::{decode, decode_into, Decoder};
pub use encode::{encode, encode_into, Encoder};

pub const ALPHABET: Alphabet<58> = match Alphabet::new(b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz") {
    Ok(alphabet) => alphabet,
    Err(_) => panic!("Could not build alphabet"),
};
