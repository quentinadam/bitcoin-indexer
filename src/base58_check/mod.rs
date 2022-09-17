mod checksum;
pub mod decode;
pub mod encode;

pub use crate::base58::{Alphabet, ALPHABET};
pub use decode::{decode, decode_into, Decoder};
pub use encode::{encode, encode_into, Encoder};

use checksum::compute_checksum;
