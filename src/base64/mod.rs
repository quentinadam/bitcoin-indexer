pub mod decode;
pub mod encode;
pub use crate::base_common::Alphabet;

pub const ALPHABET: Alphabet<64> = match Alphabet::new(b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/") {
    Ok(alphabet) => alphabet,
    Err(_) => panic!("Could not build alphabet"),
};

pub use decode::{decode, decode_into, Decoder};
pub use encode::{encode, encode_into, Encoder};
