use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::sync::Mutex;

use lazy_static::lazy_static;

use crate::engines::account::AccountEngine;
use crate::engines::models::{Transaction, TransactionId, TransactionType};
use crate::engines::transaction::TransactionEngine;

lazy_static! {
    pub static ref TRANSACTIONS: Mutex<HashMap<TransactionId, Transaction>> =
        Mutex::new(HashMap::new());
}

pub struct InMemoryTransactionEngine<A> {
    account_engine: A,
}

impl<A> TransactionEngine for InMemoryTransactionEngine<A>
where
    A: AccountEngine,
{
    fn process_transactions<I>(&self, transactions: I)
    where
        I: IntoIterator<Item = Transaction>,
    {
        for transaction in transactions {
            match transaction.transaction_type {
                TransactionType::Deposit => self.handle_deposit(&transaction),
                TransactionType::Withdrawal => self.handle_withdrawal(&transaction),
                TransactionType::Dispute => self.handle_dispute(&transaction),
                TransactionType::Resolve => self.handle_resolve(&transaction),
                TransactionType::Chargeback => self.handle_chargeback(&transaction),
            }
            match transaction.transaction_type {
                // Assumption: Anything other than deposits or withdrawals are not considered true "transactions"
                // given that their tx ID field references another transaction other than their own.
                TransactionType::Deposit | TransactionType::Withdrawal => {
                    self.commit_transaction(transaction)
                }
                _ => {}
            }
        }
    }
}

impl<A> InMemoryTransactionEngine<A>
where
    A: AccountEngine,
{
    pub fn new(account_engine: A) -> Self {
        InMemoryTransactionEngine { account_engine }
    }

    fn handle_deposit(&self, transaction: &Transaction) {
        self.account_engine
            .update_account(transaction.client_id, |account| {
                if let Some(amount) = transaction.amount {
                    account.available += amount;
                    account.total += amount;
                }
            });
    }

    fn handle_withdrawal(&self, transaction: &Transaction) {
        self.account_engine
            .update_account(transaction.client_id, |account| {
                if let Some(amount) = transaction.amount {
                    if account.available < amount {
                        return;
                    }

                    account.available -= amount;
                    account.total -= amount;
                }
            });
    }

    fn handle_dispute(&self, transaction: &Transaction) {
        let tx_to_dispute_opt = self
            .fetch_and_update_transaction_disputed_state(transaction.id, TransactionType::Dispute);

        if let Some(tx_to_dispute) = tx_to_dispute_opt {
            if let Some(amount) = tx_to_dispute.amount {
                self.account_engine
                    .update_account(tx_to_dispute.client_id, |account| {
                        account.available -= amount;
                        account.held += amount;
                    });
            }
        }
    }

    fn handle_resolve(&self, transaction: &Transaction) {
        let tx_to_resolve_opt = self
            .fetch_and_update_transaction_disputed_state(transaction.id, TransactionType::Resolve);

        if let Some(tx_to_resolve) = tx_to_resolve_opt {
            if let Some(amount) = tx_to_resolve.amount {
                self.account_engine
                    .update_account(tx_to_resolve.client_id, |account| {
                        account.available += amount;
                        account.held -= amount;
                    });
            }
        }
    }

    fn handle_chargeback(&self, transaction: &Transaction) {
        let tx_to_chargeback_opt = self.fetch_and_update_transaction_disputed_state(
            transaction.id,
            TransactionType::Chargeback,
        );

        if let Some(tx_to_chargeback) = tx_to_chargeback_opt {
            if let Some(amount) = tx_to_chargeback.amount {
                self.account_engine
                    .update_account(tx_to_chargeback.client_id, |account| {
                        account.held -= amount;
                        account.total -= amount;
                        account.locked = true;
                    });
            }
        }
    }

    fn fetch_and_update_transaction_disputed_state(
        &self,
        transaction_id: u32,
        transaction_type: TransactionType,
    ) -> Option<Transaction> {
        TRANSACTIONS
            .lock()
            .map(|mut transactions| {
                let transaction_opt = transactions.entry(transaction_id);

                return match transaction_opt {
                    Entry::Occupied(mut entry) => {
                        let transaction = entry.get_mut();

                        self.return_transaction_if_appropriate(transaction_type, transaction)
                    }
                    Entry::Vacant(_) => None,
                };
            })
            .expect("Error locking transactions")
    }

    fn return_transaction_if_appropriate(
        &self,
        transaction_type: TransactionType,
        transaction: &mut Transaction,
    ) -> Option<Transaction> {
        match transaction_type {
            TransactionType::Dispute => {
                if !transaction.is_disputed {
                    transaction.is_disputed = true;
                    return Some(transaction.clone());
                }
            }
            TransactionType::Resolve => {
                if transaction.is_disputed {
                    transaction.is_disputed = false;
                    return Some(transaction.clone());
                }
            }
            TransactionType::Chargeback => {
                if transaction.is_disputed {
                    return Some(transaction.clone());
                }
            }
            _ => return None,
        };
        None
    }

    // Assumption: The transaction ID is unique for each entry. This approach doesn't handle duplicates.
    fn commit_transaction(&self, transaction: Transaction) {
        TRANSACTIONS
            .lock()
            .map(|mut transactions| {
                transactions.insert(transaction.id, transaction);
            })
            .expect("Error locking transactions");
    }
}
