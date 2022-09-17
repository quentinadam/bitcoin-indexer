use sha2::{
    digest::{consts::U32, generic_array::GenericArray},
    Digest, Sha256,
};
use std::convert::TryInto;

fn sha256(buffer: impl AsRef<[u8]>) -> GenericArray<u8, U32> {
    let mut hasher = Sha256::new();
    hasher.update(buffer);
    let result = hasher.finalize();
    result
}

pub fn compute_checksum(buffer: impl AsRef<[u8]>) -> [u8; 4] {
    let hash = sha256(sha256(buffer));
    (&hash[0..4]).try_into().unwrap()
}
