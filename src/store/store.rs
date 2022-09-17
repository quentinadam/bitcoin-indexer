use super::{
    AugmentedTransactionStore, AugmentedTransactionStoreBackend, IndexedTransactionStore, IndexedTransactionStoreBackend,
    IntermediaryTransactionStore, IntermediaryTransactionStoreBackend, TransactionStore, TransactionStoreBackendTrait,
};
use crate::{Batcher, BlockHeader, BlockTrait, BufferWriter, HashingBufferReader, Logger, PartialLogger, SequentialThreadPool, TryInto};

#[derive(Debug)]
pub struct Store<T: TransactionStoreBackendTrait> {
    transaction_store: TransactionStore<T>,
    block_headers: Vec<BlockHeader>,
}

impl<T: TransactionStoreBackendTrait> Store<T> {
    pub fn height(&self) -> usize {
        self.block_headers.len()
    }

    pub fn backend(&self) -> &T {
        self.transaction_store.backend()
    }

    pub fn take_backend(self) -> T {
        self.transaction_store.take_backend()
    }

    pub fn transaction_store(&self) -> &TransactionStore<T> {
        &self.transaction_store
    }

    pub fn block_headers(&self) -> &Vec<BlockHeader> {
        &self.block_headers
    }

    pub fn last_block_hash(&self) -> Option<[u8; 32]> {
        match self.block_headers.last() {
            Some(block_header) => Some(block_header.hash),
            None => None,
        }
    }

    fn add_block_header(&mut self, block_header: BlockHeader) {
        if let Some(last_block_header) = self.block_headers.last() {
            assert!(last_block_header.hash == block_header.previous_block_hash);
        }
        self.block_headers.push(block_header);
    }

    pub fn add_block(&mut self, block: &impl BlockTrait) {
        self.add_block_header(block.header().clone());
        self.transaction_store.add_block(block);
    }
}

pub type IndexedStore = Store<IndexedTransactionStoreBackend>;

fn process_blocks(store: &mut Store<impl TransactionStoreBackendTrait>, blocks: &[impl BlockTrait], logger: Logger) {
    let mut logger = PartialLogger::new(1000, &logger);
    for block in blocks {
        let interval = logger.interval();
        logger.log(|_| format!("processing blocks {} - {}...", block.height(), block.height() + interval - 1));
        store.add_block(block);
    }
}

impl IndexedStore {
    pub fn from_blocks<T: 'static + BlockTrait + Clone + Send + Sync>(
        blocks: Vec<T>,
        threads: usize,
        batch_size: usize,
        logger: Logger,
    ) -> Self {
        let mut store = Self::large();
        if threads > 1 {
            let batcher = Batcher::new(blocks, batch_size);
            let threadpool = SequentialThreadPool::new(
                threads,
                move |blocks: Vec<T>| {
                    let mut store = IntermediaryStore::new();
                    process_blocks(&mut store, &blocks, logger);
                    (store, blocks)
                },
                batcher,
            );
            for (intermediary_store, blocks) in threadpool {
                logger.log(format!(
                    "merging blocks {} - {}...",
                    blocks[0].height(),
                    blocks[blocks.len() - 1].height(),
                ));
                intermediary_store.merge(&mut store);
                logger.log(format!(
                    "merging blocks {} - {} done!",
                    blocks[0].height(),
                    blocks[blocks.len() - 1].height()
                ));
            }
        } else {
            process_blocks(&mut store, &blocks, logger);
        }
        store
    }

    pub fn from_file(path: &str, logger: &Logger) -> Option<Self> {
        logger.log("reading store from file...");
        let store = match std::fs::read(path) {
            Ok(buffer) => {
                let mut reader = HashingBufferReader::new(&buffer);
                Some(Self::from_reader(&mut reader, logger))
            }
            Err(_) => None,
        };
        logger.log("reading store from file done!");
        store
    }

    fn from_reader(reader: &mut HashingBufferReader, logger: &Logger) -> Self {
        let mut block_headers = Vec::new();
        for _ in 0..reader.read_u32_le(&mut None) {
            let hash = reader.read_hash(&mut None);
            let previous_block_hash = reader.read_hash(&mut None);
            block_headers.push(BlockHeader { hash, previous_block_hash });
        }
        Self {
            block_headers,
            transaction_store: IndexedTransactionStore::from_reader(reader, logger),
        }
    }

    fn to_writer(&self, writer: &mut BufferWriter, logger: &Logger) {
        writer.write_u32(self.block_headers().len().try_into().unwrap());
        for block_header in self.block_headers().iter() {
            writer.write_buffer(&block_header.hash);
            writer.write_buffer(&block_header.previous_block_hash);
        }
        self.backend().to_writer(writer, logger);
    }

    pub fn to_file(&self, path: &str, logger: &Logger) {
        logger.log("writing store to file...");
        let mut writer = BufferWriter::new();
        self.to_writer(&mut writer, logger);
        logger.log("writing buffer to file...");
        std::fs::write(path, writer.buffer()).unwrap();
        logger.log("writing buffer to file done!");
        logger.log("writing store to file done!");
    }

    pub fn large() -> Self {
        Self {
            block_headers: Vec::new(),
            transaction_store: IndexedTransactionStore::large(),
        }
    }
}

pub type IntermediaryStore = Store<IntermediaryTransactionStoreBackend>;

impl IntermediaryStore {
    pub fn new() -> Self {
        Self {
            block_headers: Vec::new(),
            transaction_store: IntermediaryTransactionStore::new(),
        }
    }

    fn merge<T: TransactionStoreBackendTrait>(&self, store: &mut Store<T>) {
        for block_header in &self.block_headers {
            store.add_block_header(block_header.clone());
        }
        self.transaction_store.merge(&mut store.transaction_store);
    }
}

pub type AugmentedStore<'a, T> = Store<AugmentedTransactionStoreBackend<'a, T>>;

impl<'a, T: TransactionStoreBackendTrait> AugmentedStore<'a, T> {
    pub fn new(base_store: &'a Store<T>) -> Self {
        Self {
            block_headers: Vec::new(),
            transaction_store: AugmentedTransactionStore::new(base_store.transaction_store()),
        }
    }
}
