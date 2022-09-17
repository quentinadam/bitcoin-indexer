use super::{
    AugmentedTransactionStoreBackend, IndexedTransactionStoreBackend, IntermediaryTransactionStoreBackend,
    ReadonlyTransactionStoreBackendTrait, TransactionStoreAugmentation, TransactionStoreBackendTrait,
};
use crate::{Address, BlockTrait, HashingBufferReader, Logger, Transaction, TransactionOutput, TryInto};

trait ScriptExt {
    #[allow(non_snake_case)]
    fn starts_with_OP_RETURN(&self) -> bool;
}

impl ScriptExt for Vec<u8> {
    fn starts_with_OP_RETURN(&self) -> bool {
        self.len() > 0 && self[0] == 0x6a
    }
}

#[derive(Debug)]
pub struct TransactionStore<T: TransactionStoreBackendTrait> {
    strict: bool,
    backend: T,
}

impl<T: TransactionStoreBackendTrait> TransactionStore<T> {
    pub fn backend(&self) -> &T {
        &self.backend
    }

    pub fn mut_backend(&mut self) -> &mut T {
        &mut self.backend
    }

    pub fn take_backend(self) -> T {
        self.backend
    }

    fn spend_transaction_outputs(&mut self, txos: &[TransactionOutput]) {
        for txo in txos {
            if txo != &TransactionOutput::new([0; 32], u32::MAX) {
                assert!(self.backend.spend_transaction_output(txo) || !self.strict);
            }
        }
    }

    pub fn can_add_transaction(&self, transaction: &Transaction) -> bool {
        for txo in &transaction.inputs {
            if !self.backend.has_transaction_output(txo) {
                return false;
            }
        }
        true
    }

    pub fn add_transaction(&mut self, transaction: &Transaction) {
        self.spend_transaction_outputs(&transaction.inputs);
        for (index, output) in transaction.outputs.iter().enumerate() {
            if !output.script.starts_with_OP_RETURN() {
                let index = index.try_into().unwrap();
                let address = Address::from_script(&output.script).ok();
                self.backend
                    .add_transaction_output(TransactionOutput::new(transaction.hash, index), address, output.value);
            }
        }
    }

    pub fn add_block(&mut self, block: &impl BlockTrait) {
        block.transactions(&mut |transaction| {
            self.add_transaction(transaction);
        });
    }
}

pub type IndexedTransactionStore = TransactionStore<IndexedTransactionStoreBackend>;

impl IndexedTransactionStore {
    pub fn new(strict: bool) -> Self {
        Self {
            strict,
            backend: IndexedTransactionStoreBackend::new(),
        }
    }

    pub fn large() -> Self {
        Self {
            strict: true,
            backend: IndexedTransactionStoreBackend::large(),
        }
    }

    pub fn from_reader(reader: &mut HashingBufferReader, logger: &Logger) -> Self {
        Self {
            strict: true,
            backend: IndexedTransactionStoreBackend::from_reader(reader, logger),
        }
    }
}

pub type IntermediaryTransactionStore = TransactionStore<IntermediaryTransactionStoreBackend>;

impl IntermediaryTransactionStore {
    pub fn new() -> Self {
        Self {
            strict: false,
            backend: IntermediaryTransactionStoreBackend::new(),
        }
    }

    pub fn merge<T: TransactionStoreBackendTrait>(&self, store: &mut TransactionStore<T>) {
        store.spend_transaction_outputs(&self.backend().spent_txos());
        for (txo, (address, value)) in self.backend().unspent_txos() {
            store.mut_backend().add_transaction_output(txo.clone(), address.clone(), *value);
        }
    }
}

pub type AugmentedTransactionStore<'a, T> = TransactionStore<AugmentedTransactionStoreBackend<'a, T>>;

impl<'a, T: TransactionStoreBackendTrait> AugmentedTransactionStore<'a, T> {
    pub fn new(transaction_store: &'a TransactionStore<T>) -> Self {
        Self {
            strict: true,
            backend: AugmentedTransactionStoreBackend::new(transaction_store.backend()),
        }
    }

    pub fn augmentation(&self) -> &TransactionStoreAugmentation {
        &self.backend().augmentation()
    }

    pub fn take_augmentation(self) -> TransactionStoreAugmentation {
        self.take_backend().take_augmentation()
    }
}

#[derive(Debug)]
pub struct ReadonlyTransactionStore<T: ReadonlyTransactionStoreBackendTrait> {
    backend: T,
}

impl<T: ReadonlyTransactionStoreBackendTrait> ReadonlyTransactionStore<T> {
    pub fn new(backend: T) -> Self {
        Self { backend }
    }
}

impl<T: ReadonlyTransactionStoreBackendTrait> ReadonlyTransactionStore<T> {
    pub fn balance(&self, address: &Address) -> u64 {
        self.backend.balance(address)
    }

    pub fn iterate_transaction_outputs(&self, address: &Address, callback: impl FnMut(&TransactionOutput, u64)) {
        self.backend.iterate_transaction_outputs(address, callback);
    }
}
