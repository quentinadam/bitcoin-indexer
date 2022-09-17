#[derive(Eq, PartialEq, Hash, Debug, Clone)]
pub struct TransactionOutput {
    pub hash: [u8; 32],
    pub index: u32,
}

impl TransactionOutput {
    pub fn new(hash: [u8; 32], index: u32) -> Self {
        return Self { hash, index };
    }
}
