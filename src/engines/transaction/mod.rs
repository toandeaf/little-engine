pub use in_memory_transaction::InMemoryTransactionEngine;

use crate::engines::models::Transaction;

mod in_memory_transaction;

pub trait TransactionEngine {
    fn process_transactions<I>(&self, transactions: I)
    where
        I: IntoIterator<Item = Transaction>;
}
