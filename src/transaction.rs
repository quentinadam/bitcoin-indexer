use crate::{Hasher, HashingBufferReader, TransactionOutput, TryInto};

#[derive(Debug, Clone)]
pub struct Output {
    pub value: u64,
    pub script: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct Transaction {
    pub hash: [u8; 32],
    pub inputs: Vec<TransactionOutput>,
    pub outputs: Vec<Output>,
}

impl Transaction {
    pub fn from_slice(buffer: &[u8]) -> Self {
        let mut reader = HashingBufferReader::new(buffer);
        Self::from_reader(&mut reader)
    }

    pub fn from_reader(reader: &mut HashingBufferReader) -> Self {
        let mut hasher = Hasher::new();
        let _version = reader.read_i32_le(&mut Some(&mut hasher));
        let mut flags = 0;
        if reader.peek_u8() == 0 {
            reader.skip(1, &mut None);
            flags = reader.read_u8(&mut None);
        }
        let count_inputs = reader.read_var_int_le(&mut Some(&mut hasher));
        let mut inputs = Vec::with_capacity(count_inputs.try_into().unwrap());
        for _ in 0..count_inputs {
            let hash = reader.read_hash(&mut Some(&mut hasher));
            let index = reader.read_u32_le(&mut Some(&mut hasher));
            let _script = reader.read_var_buffer_le(&mut Some(&mut hasher));
            let _sequence = reader.read_u32_le(&mut Some(&mut hasher));
            inputs.push(TransactionOutput::new(hash, index));
        }
        let count_outputs = reader.read_var_int_le(&mut Some(&mut hasher));
        let mut outputs = Vec::with_capacity(count_outputs.try_into().unwrap());
        for _ in 0..count_outputs {
            let value = reader.read_u64_le(&mut Some(&mut hasher));
            let script = reader.read_var_buffer_le(&mut Some(&mut hasher));
            outputs.push(Output { value, script });
        }
        if (flags & 0x01) != 0 {
            for _ in 0..count_inputs {
                let count_witnesses = reader.read_var_int_le(&mut None);
                for _ in 0..count_witnesses {
                    reader.read_var_buffer_le(&mut None);
                }
            }
        }
        let _locktime = reader.read_u32_le(&mut Some(&mut hasher));
        Self {
            hash: hasher.digest(),
            inputs,
            outputs,
        }
    }
}
