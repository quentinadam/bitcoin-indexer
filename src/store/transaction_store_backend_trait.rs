use crate::{Address, TransactionOutput};

pub trait ReadonlyTransactionStoreBackendTrait {
    fn iterate_transaction_outputs(&self, address: &Address, callback: impl FnMut(&TransactionOutput, u64));

    fn balance(&self, address: &Address) -> u64 {
        let mut sum = 0;
        self.iterate_transaction_outputs(address, |_, value| {
            sum += value;
        });
        sum
    }
}

pub trait TransactionStoreBackendTrait {
    fn has_transaction_output(&self, txo: &TransactionOutput) -> bool;
    fn spend_transaction_output(&mut self, txo: &TransactionOutput) -> bool;
    fn add_transaction_output(&mut self, txo: TransactionOutput, address: Option<Address>, value: u64);
}
