use super::{ReadonlyTransactionStoreBackendTrait, TransactionStoreBackendTrait};
use crate::{
    Address, AddressHashMap, BufferWriter, HashSet, HashingBufferReader, Logger, PartialLogger, TransactionOutput,
    TransactionOutputHashMap, TryInto,
};

#[derive(Debug)]
pub struct IndexedTransactionStoreBackend {
    unspent_txo_address_map: TransactionOutputHashMap<(u64, Option<Address>)>,
    address_unspent_txos_map: AddressHashMap<TransactionOutputHashMap<u64>>,
}

impl IndexedTransactionStoreBackend {
    pub fn new() -> Self {
        Self {
            unspent_txo_address_map: TransactionOutputHashMap::new(),
            address_unspent_txos_map: AddressHashMap::new(),
        }
    }

    pub fn large() -> Self {
        Self {
            unspent_txo_address_map: TransactionOutputHashMap::with_capacity(100_000_000),
            address_unspent_txos_map: AddressHashMap::with_capacity(50_000_000),
        }
    }

    pub fn from_reader(reader: &mut HashingBufferReader, logger: &Logger) -> Self {
        let mut store = Self::large();
        let mut logger = PartialLogger::new(1000000, logger);
        for _ in 0..reader.read_u32_le(&mut None) {
            logger.log(|index| format!("reading utxo {} from buffer...", index));
            let txo = TransactionOutput::new(reader.read_hash(&mut None), reader.read_u32_le(&mut None));
            let value = reader.read_u64_le(&mut None);
            let address = match reader.read_bool(&mut None) {
                true => Some(Address::from_slice(reader.read_buffer(21, &mut None)).unwrap()),
                false => None,
            };
            store.add_transaction_output(txo, address, value);
        }
        store
    }

    pub fn to_writer(&self, writer: &mut BufferWriter, logger: &Logger) {
        let mut logger = PartialLogger::new(1000000, logger);
        writer.write_u32(self.unspent_txo_address_map.len().try_into().unwrap());
        for (txo, (value, address)) in &self.unspent_txo_address_map {
            logger.log(|index| format!("writing utxo {} to buffer...", index));
            writer.write_buffer(&txo.hash);
            writer.write_u32(txo.index);
            writer.write_u64(*value);
            match address {
                Some(address) => {
                    writer.write_u8(1);
                    writer.write_buffer(&address.to_vec());
                }
                None => {
                    writer.write_u8(0);
                }
            }
        }
    }
}

impl ReadonlyTransactionStoreBackendTrait for IndexedTransactionStoreBackend {
    fn iterate_transaction_outputs(&self, address: &Address, mut callback: impl FnMut(&TransactionOutput, u64)) {
        if let Some(txos) = self.address_unspent_txos_map.get(address) {
            for (txo, value) in txos {
                callback(txo, *value);
            }
        }
    }
}

impl TransactionStoreBackendTrait for IndexedTransactionStoreBackend {
    fn has_transaction_output(&self, txo: &TransactionOutput) -> bool {
        self.unspent_txo_address_map.get(txo).is_some()
    }

    fn spend_transaction_output(&mut self, txo: &TransactionOutput) -> bool {
        match self.unspent_txo_address_map.remove(txo) {
            Some((_, address)) => {
                if let Some(address) = address {
                    let unspent_txos = self.address_unspent_txos_map.get_mut(&address).unwrap();
                    unspent_txos.remove(txo);
                    if unspent_txos.len() == 0 {
                        self.address_unspent_txos_map.remove(&address);
                    }
                }
                true
            }
            None => false,
        }
    }

    fn add_transaction_output(&mut self, txo: TransactionOutput, address: Option<Address>, value: u64) {
        self.unspent_txo_address_map.insert(txo.clone(), (value, address.clone()));
        if let Some(address) = address {
            match self.address_unspent_txos_map.get_mut(&address) {
                Some(unspent_txos) => {
                    unspent_txos.insert(txo.clone(), value);
                }
                None => {
                    let mut unspent_txos = TransactionOutputHashMap::new();
                    unspent_txos.insert(txo.clone(), value);
                    self.address_unspent_txos_map.insert(address.clone(), unspent_txos);
                }
            };
        }
    }
}

#[derive(Debug)]
pub struct IntermediaryTransactionStoreBackend {
    spent_txos: Vec<TransactionOutput>,
    unspent_txos: TransactionOutputHashMap<(Option<Address>, u64)>,
}

impl IntermediaryTransactionStoreBackend {
    pub fn new() -> Self {
        Self {
            spent_txos: Vec::new(),
            unspent_txos: TransactionOutputHashMap::new(),
        }
    }

    pub fn spent_txos(&self) -> &Vec<TransactionOutput> {
        &self.spent_txos
    }

    pub fn unspent_txos(&self) -> &TransactionOutputHashMap<(Option<Address>, u64)> {
        &self.unspent_txos
    }
}

impl TransactionStoreBackendTrait for IntermediaryTransactionStoreBackend {
    fn has_transaction_output(&self, txo: &TransactionOutput) -> bool {
        self.unspent_txos.get(txo).is_some()
    }

    fn spend_transaction_output(&mut self, txo: &TransactionOutput) -> bool {
        match self.unspent_txos.remove(txo) {
            Some(_) => true,
            None => {
                self.spent_txos.push(txo.clone());
                false
            }
        }
    }

    fn add_transaction_output(&mut self, txo: TransactionOutput, address: Option<Address>, value: u64) {
        self.unspent_txos.insert(txo, (address, value));
    }
}

pub struct TransactionStoreAugmentation {
    spent_txos: HashSet<TransactionOutput>,
    store: IndexedTransactionStoreBackend,
}

impl TransactionStoreAugmentation {
    fn new() -> Self {
        Self {
            spent_txos: HashSet::new(),
            store: IndexedTransactionStoreBackend::new(),
        }
    }
}

pub struct AugmentedTransactionStoreBackend<'a, T: TransactionStoreBackendTrait> {
    base_store: &'a T,
    store: TransactionStoreAugmentation,
}

impl<'a, T: TransactionStoreBackendTrait> AugmentedTransactionStoreBackend<'a, T> {
    pub fn new(base_store: &'a T) -> Self {
        Self {
            base_store,
            store: TransactionStoreAugmentation::new(),
        }
    }

    pub fn augmentation(&self) -> &TransactionStoreAugmentation {
        &self.store
    }

    pub fn take_augmentation(self) -> TransactionStoreAugmentation {
        self.store
    }
}

impl<'a, T: TransactionStoreBackendTrait> TransactionStoreBackendTrait for AugmentedTransactionStoreBackend<'a, T> {
    fn has_transaction_output(&self, txo: &TransactionOutput) -> bool {
        self.store.store.has_transaction_output(txo)
            || (self.store.spent_txos.get(txo).is_none() && self.base_store.has_transaction_output(txo))
    }

    fn spend_transaction_output(&mut self, txo: &TransactionOutput) -> bool {
        if self.store.store.spend_transaction_output(txo) {
            true
        } else {
            self.base_store.has_transaction_output(txo) && self.store.spent_txos.insert(txo.clone())
        }
    }

    fn add_transaction_output(&mut self, txo: TransactionOutput, address: Option<Address>, value: u64) {
        self.store.store.add_transaction_output(txo, address, value);
    }
}

pub struct ReadonlyAugmentedTransactionStoreBackend<'a, T: ReadonlyTransactionStoreBackendTrait> {
    store: &'a T,
    augmentation: Option<&'a TransactionStoreAugmentation>,
}

impl<'a, T: ReadonlyTransactionStoreBackendTrait> ReadonlyAugmentedTransactionStoreBackend<'a, T> {
    pub fn new(store: &'a T, augmentation: Option<&'a TransactionStoreAugmentation>) -> Self {
        Self { store, augmentation }
    }
}

impl<'a, T: ReadonlyTransactionStoreBackendTrait> ReadonlyTransactionStoreBackendTrait for ReadonlyAugmentedTransactionStoreBackend<'a, T> {
    fn iterate_transaction_outputs(&self, address: &Address, mut callback: impl FnMut(&TransactionOutput, u64)) {
        match self.augmentation {
            Some(augmentation) => {
                self.store.iterate_transaction_outputs(address, |txo, value| {
                    if !augmentation.spent_txos.contains(txo) {
                        callback(txo, value);
                    }
                });
                augmentation.store.iterate_transaction_outputs(address, callback);
            }
            None => self.store.iterate_transaction_outputs(address, callback),
        }
    }
}
