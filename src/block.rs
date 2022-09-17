use crate::{Hasher, HashingBufferReader, Transaction, TryInto};

#[derive(Debug, Clone)]
pub struct BlockHeader {
    pub hash: [u8; 32],
    pub previous_block_hash: [u8; 32],
}

impl BlockHeader {
    pub fn from_buffer(buffer: &[u8]) -> Self {
        let mut hasher = Hasher::new();
        let mut reader = HashingBufferReader::new(buffer);
        let buffer = reader.read_buffer(80, &mut Some(&mut hasher));
        let hash = hasher.digest();
        let mut reader = HashingBufferReader::new(buffer);
        reader.skip(4, &mut None);
        let previous_block_hash = reader.read_hash(&mut None);
        Self { hash, previous_block_hash }
    }
}

pub fn iterate_transactions<F: FnMut(Transaction)>(buffer: &[u8], callback: &mut F) -> () {
    let mut reader = HashingBufferReader::new(&buffer[80..]);
    let count: usize = reader.read_var_int_le(&mut None).try_into().unwrap();
    for _ in 0..count {
        let transaction = Transaction::from_reader(&mut reader);
        callback(transaction);
    }
}

pub trait BlockTrait {
    fn header(&self) -> &BlockHeader;

    fn hash(&self) -> [u8; 32] {
        self.header().hash
    }

    fn previous_block_hash(&self) -> [u8; 32] {
        self.header().previous_block_hash
    }

    fn height(&self) -> usize;

    fn transactions<F: FnMut(&Transaction)>(&self, callback: &mut F) -> ();
}

#[derive(Debug, Clone)]
pub struct Block {
    height: usize,
    header: BlockHeader,
    transactions: Vec<Transaction>,
}

impl Block {
    pub fn new(buffer: &[u8], height: usize) -> Self {
        let header = BlockHeader::from_buffer(&buffer);
        let mut transactions = Vec::new();
        iterate_transactions(&buffer, &mut |transaction| {
            transactions.push(transaction);
        });
        Self {
            height,
            header,
            transactions,
        }
    }
}

impl BlockTrait for Block {
    fn header(&self) -> &BlockHeader {
        &self.header
    }

    fn transactions<F: FnMut(&Transaction)>(&self, callback: &mut F) -> () {
        for transaction in &self.transactions {
            callback(transaction);
        }
    }

    fn height(&self) -> usize {
        self.height
    }
}
