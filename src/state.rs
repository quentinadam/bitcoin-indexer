use crate::store::{ReadonlyAugmentedTransactionStoreBackend, ReadonlyTransactionStore};
use crate::{
    Address, Arc, AugmentedStore, Block, Client, IndexedStore, IndexedTransactionStoreBackend, LastBlocks, Logger, Mutex, RwLock,
    TransactionOutput, TransactionStoreAugmentation,
};

enum Update {
    LastAugmentationUpdate(TransactionStoreAugmentation),
    AugmentationsUpdate(Vec<TransactionStoreAugmentation>, Vec<Block>),
}

pub struct State {
    store: Arc<RwLock<IndexedStore>>,
    tail_blocks: Mutex<LastBlocks>,
    augmentations: RwLock<Vec<TransactionStoreAugmentation>>,
    client: Client,
    mutex: Mutex<()>,
    confirmations: usize,
}

impl State {
    pub fn new(store: IndexedStore, client: Client, confirmations: usize) -> Self {
        Self {
            augmentations: RwLock::new(Vec::new()),
            store: Arc::new(RwLock::new(store)),
            tail_blocks: Mutex::new(LastBlocks::new()),
            client,
            mutex: Mutex::new(()),
            confirmations,
        }
    }

    pub fn confirmations(&self) -> usize {
        self.confirmations
    }

    async fn compute_update(&self, update_store: bool, logger: &Logger) -> Update {
        let store = self.store.read().await;
        let mut tail_blocks = self.tail_blocks.lock().await;
        tail_blocks.update_mempool(&self.client, logger).await;
        if tail_blocks.update_blocks(&store, &self.client, logger).await {
            let blocks = if update_store {
                tail_blocks.pop(self.confirmations)
            } else {
                Vec::new()
            };
            let augmentations = if blocks.len() > 0 {
                let mut store = AugmentedStore::new(&store);
                for block in &blocks {
                    store.add_block(block);
                }
                tail_blocks.augmentations(&store)
            } else {
                tail_blocks.augmentations(&store)
            };
            assert!(augmentations.len() == self.confirmations);
            Update::AugmentationsUpdate(augmentations, blocks)
        } else {
            Update::LastAugmentationUpdate(tail_blocks.last_augmentation(&store))
        }
    }

    async fn apply_update(&self, update: Update) -> bool {
        let mut augmentations = self.augmentations.write().await;
        match update {
            Update::LastAugmentationUpdate(augmentation) => {
                let index = augmentations.len() - 1;
                augmentations[index] = augmentation;
                false
            }
            Update::AugmentationsUpdate(updated_augmentations, blocks) => {
                *augmentations = updated_augmentations;
                if blocks.len() > 0 {
                    let store = &mut *self.store.write().await;
                    for block in &blocks {
                        store.add_block(block);
                    }
                    true
                } else {
                    false
                }
            }
        }
    }

    pub async fn update(&self) -> bool {
        let mutex = self.mutex.try_lock();
        let logger = Logger::new();
        logger.log("computing update...");
        let update = self.compute_update(mutex.is_ok(), &logger).await;
        logger.log("computing update done!");
        let logger = Logger::new();
        logger.log("applying update...");
        let result = self.apply_update(update).await;
        logger.log("applying update done!");
        result
    }

    fn augmented_store<'a>(
        &self,
        store: &'a IndexedStore,
        augmentations: &'a Vec<TransactionStoreAugmentation>,
        confirmations: usize,
    ) -> ReadonlyTransactionStore<ReadonlyAugmentedTransactionStoreBackend<'a, IndexedTransactionStoreBackend>> {
        if confirmations == augmentations.len() {
            ReadonlyTransactionStore::new(ReadonlyAugmentedTransactionStoreBackend::new(store.backend(), None))
        } else {
            assert!(confirmations < augmentations.len());
            let augmentation = &augmentations[augmentations.len() - 1 - confirmations];
            ReadonlyTransactionStore::new(ReadonlyAugmentedTransactionStoreBackend::new(store.backend(), Some(augmentation)))
        }
    }

    pub async fn iterate_transaction_outputs(
        &self,
        address: &Address,
        confirmations: usize,
        callback: impl FnMut(&TransactionOutput, u64),
    ) {
        let augmentations = self.augmentations.read().await;
        let store = self.store.read().await;
        let store = self.augmented_store(&store, &augmentations, confirmations);
        store.iterate_transaction_outputs(address, callback);
    }

    pub async fn transaction_outputs_array(
        &self,
        addresses: Vec<Address>,
        confirmations: usize,
    ) -> Vec<(Address, Vec<(TransactionOutput, u64)>)> {
        let augmentations = self.augmentations.read().await;
        let store = self.store.read().await;
        let store = self.augmented_store(&store, &augmentations, confirmations);
        addresses
            .into_iter()
            .map(|address| {
                let mut utxos = Vec::new();
                store.iterate_transaction_outputs(&address, |utxo, value| {
                    utxos.push((utxo.clone(), value));
                });
                (address, utxos)
            })
            .collect()
    }

    pub async fn balance(&self, address: &Address, confirmations: usize) -> u64 {
        let augmentations = self.augmentations.read().await;
        let store = self.store.read().await;
        let store = self.augmented_store(&store, &augmentations, confirmations);
        store.balance(address)
    }

    pub async fn balance_array(&self, addresses: Vec<Address>, confirmations: usize) -> Vec<(Address, u64)> {
        let augmentations = self.augmentations.read().await;
        let store = self.store.read().await;
        let store = self.augmented_store(&store, &augmentations, confirmations);
        addresses
            .into_iter()
            .map(|address| {
                let balance = store.balance(&address);
                (address, balance)
            })
            .collect()
    }

    pub async fn write(&self, file: &str) {
        let _ = self.mutex.lock().await;
        let store = self.store.read().await;
        let logger = Logger::new();
        store.to_file(file, &logger);
    }
}
