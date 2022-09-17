use crate::TryInto;
use sha2::{Digest, Sha256};

pub struct Hasher {
    hasher: Sha256,
}

impl Hasher {
    #[inline(always)]
    pub fn new() -> Self {
        Self { hasher: Sha256::new() }
    }

    #[inline(always)]
    pub fn update(&mut self, buffer: &[u8]) {
        self.hasher.update(buffer);
    }

    #[inline(always)]
    pub fn digest(self) -> [u8; 32] {
        Sha256::digest(&self.hasher.finalize()).try_into().unwrap()
    }
}

pub struct HashingBufferReader<'a> {
    buffer: &'a [u8],
    offset: usize,
}

impl<'a> HashingBufferReader<'a> {
    #[inline(always)]
    pub fn new(buffer: &'a [u8]) -> HashingBufferReader<'a> {
        Self { buffer, offset: 0 }
    }

    #[inline(always)]
    pub fn offset(&self) -> usize {
        self.offset
    }

    #[inline(always)]
    pub fn read_i32_le(&mut self, hasher: &mut Option<&mut Hasher>) -> i32 {
        i32::from_le_bytes(self.read_buffer(4, hasher).try_into().unwrap())
    }

    #[inline(always)]
    pub fn read_bool(&mut self, hasher: &mut Option<&mut Hasher>) -> bool {
        match self.read_buffer(1, hasher)[0] {
            0 => false,
            _ => true,
        }
    }

    #[inline(always)]
    pub fn read_u8(&mut self, hasher: &mut Option<&mut Hasher>) -> u8 {
        self.read_buffer(1, hasher)[0]
    }

    #[inline(always)]
    pub fn read_u16_le(&mut self, hasher: &mut Option<&mut Hasher>) -> u16 {
        u16::from_le_bytes(self.read_buffer(2, hasher).try_into().unwrap())
    }

    #[inline(always)]
    pub fn read_u32_le(&mut self, hasher: &mut Option<&mut Hasher>) -> u32 {
        u32::from_le_bytes(self.read_buffer(4, hasher).try_into().unwrap())
    }

    #[inline(always)]
    pub fn read_u64_le(&mut self, hasher: &mut Option<&mut Hasher>) -> u64 {
        u64::from_le_bytes(self.read_buffer(8, hasher).try_into().unwrap())
    }

    #[inline(always)]
    pub fn read_var_int_le(&mut self, hasher: &mut Option<&mut Hasher>) -> u64 {
        match self.read_u8(hasher) {
            0xFD => self.read_u16_le(hasher).into(),
            0xFE => self.read_u32_le(hasher).into(),
            0xFF => self.read_u64_le(hasher).into(),
            byte => byte.into(),
        }
    }

    #[inline(always)]
    pub fn read_buffer(&mut self, length: usize, hasher: &mut Option<&mut Hasher>) -> &'a [u8] {
        let offset = self.offset;
        self.offset += length;
        let buffer = &self.buffer[offset..self.offset];
        if let Some(hasher) = hasher {
            hasher.update(buffer);
        }
        buffer
    }

    #[inline(always)]
    pub fn read_var_buffer_le(&mut self, hasher: &mut Option<&mut Hasher>) -> Vec<u8> {
        let length = self.read_var_int_le(hasher);
        self.read_buffer(length.try_into().unwrap(), hasher).to_vec()
    }

    #[inline(always)]
    pub fn read_hash(&mut self, hasher: &mut Option<&mut Hasher>) -> [u8; 32] {
        self.read_buffer(32, hasher).try_into().unwrap()
    }

    #[inline(always)]
    pub fn peek_u8(&self) -> u8 {
        self.buffer[self.offset]
    }

    #[inline(always)]
    pub fn skip(&mut self, length: usize, hasher: &mut Option<&mut Hasher>) {
        self.read_buffer(length, hasher);
    }
}
