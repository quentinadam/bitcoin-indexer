pub struct BufferWriter {
    buffer: Vec<u8>,
}

impl BufferWriter {
    pub fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    pub fn write_u8(&mut self, value: u8) {
        self.write_buffer(&[value]);
    }

    pub fn write_u32(&mut self, value: u32) {
        self.write_buffer(&value.to_le_bytes());
    }

    pub fn write_u64(&mut self, value: u64) {
        self.write_buffer(&value.to_le_bytes());
    }

    pub fn write_buffer(&mut self, value: &[u8]) {
        self.buffer.extend_from_slice(value);
    }

    pub fn write_hash(&mut self, value: [u8; 32]) {
        self.write_buffer(&value);
    }

    pub fn buffer(self) -> Vec<u8> {
        self.buffer
    }
}
