use crate::{
    reverse_hex, AugmentedTransactionStore, Block, BlockTrait, Client, HashMap, IndexedStore, Logger, Store, Transaction,
    TransactionStoreAugmentation, TransactionStoreBackendTrait, VecDeque,
};

#[derive(Debug)]
struct Mempool {
    pub transactions: HashMap<[u8; 32], Transaction>,
    pub hashes: Vec<[u8; 32]>,
}

impl Mempool {
    pub fn new() -> Self {
        Self {
            transactions: HashMap::new(),
            hashes: Vec::new(),
        }
    }

    pub fn transactions<F: FnMut(&Transaction)>(&self, callback: &mut F) {
        for hash in self.hashes.iter() {
            if let Some(transaction) = self.transactions.get(hash) {
                callback(transaction);
            }
        }
    }
}

struct BlockUpdater<'a> {
    store: &'a IndexedStore,
    blocks: &'a mut VecDeque<Block>,
}

impl<'a> BlockUpdater<'a> {
    fn height(&self) -> usize {
        self.store.height() + self.blocks.len()
    }

    fn last_block_hash(&self) -> [u8; 32] {
        match self.blocks.len() {
            0 => self.store.last_block_hash().unwrap(),
            n => self.blocks[n - 1].hash(),
        }
    }

    fn add_block(&mut self, block: Block) {
        if block.previous_block_hash() == self.last_block_hash() {
            self.blocks.push_back(block);
        } else {
            assert!(self.blocks.pop_back().is_some());
        }
    }

    async fn next_block(&self, client: &Client, logger: &Logger) -> Option<Block> {
        let hash = client.getblockhash(self.height(), logger).await?;
        let buffer = client.getblock(hash, logger).await?;
        let block = Block::new(&buffer, self.height());
        Some(block)
    }

    async fn update(&mut self, client: &Client, logger: &Logger) -> bool {
        let mut updated = false;
        while let Some(block) = self.next_block(client, logger).await {
            updated = true;
            self.add_block(block);
        }
        updated
    }
}

pub struct LastBlocks {
    blocks: VecDeque<Block>,
    mempool: Mempool,
}

impl LastBlocks {
    pub fn new() -> Self {
        Self {
            blocks: VecDeque::new(),
            mempool: Mempool::new(),
        }
    }

    fn augmentation(&self, store: &Store<impl TransactionStoreBackendTrait>, count: usize) -> TransactionStoreAugmentation {
        assert!(count <= self.blocks.len());
        let mut augmented_store = AugmentedTransactionStore::new(store.transaction_store());
        for i in 0..count + 1 {
            if i < self.blocks.len() {
                augmented_store.add_block(&self.blocks[i]);
            } else {
                self.mempool.transactions(&mut |transaction| {
                    if augmented_store.can_add_transaction(transaction) {
                        augmented_store.add_transaction(transaction)
                    }
                });
            }
        }
        augmented_store.take_augmentation()
    }

    pub fn len(&self) -> usize {
        self.blocks.len() + 1
    }

    pub fn pop(&mut self, confirmations: usize) -> Vec<Block> {
        let mut blocks = Vec::new();
        while self.blocks.len() > confirmations - 1 {
            blocks.push(self.blocks.pop_front().unwrap());
        }
        blocks
    }

    pub fn last_augmentation(&self, store: &Store<impl TransactionStoreBackendTrait>) -> TransactionStoreAugmentation {
        self.augmentation(store, self.blocks.len())
    }

    pub fn augmentations(&self, store: &Store<impl TransactionStoreBackendTrait>) -> Vec<TransactionStoreAugmentation> {
        let mut augmentations = Vec::new();
        for i in 0..self.blocks.len() + 1 {
            augmentations.push(self.augmentation(store, i));
        }
        augmentations
    }

    pub async fn update_blocks(&mut self, store: &IndexedStore, client: &Client, logger: &Logger) -> bool {
        let mut updater = BlockUpdater {
            store,
            blocks: &mut self.blocks,
        };
        updater.update(client, logger).await
    }

    pub async fn update_mempool(&mut self, client: &Client, logger: &Logger) {
        let hashes = client.getrawmempool(logger).await;
        let mut transactions = HashMap::new();
        for hash in &hashes {
            match self.mempool.transactions.remove(hash) {
                Some(transaction) => {
                    transactions.insert(*hash, transaction);
                }
                None => match client.getrawtransaction(&hash, logger).await {
                    Some(transaction) => {
                        transactions.insert(*hash, transaction);
                    }
                    None => logger.log(format!("Could not get transaction {}", reverse_hex::encode(hash))),
                },
            }
        }
        self.mempool = Mempool { hashes, transactions };
    }
}
