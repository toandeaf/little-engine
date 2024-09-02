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
            commit_transaction(transaction);
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
                    account.available -= amount;
                    account.total -= amount;
                }
            });
    }

    fn handle_dispute(&self, transaction: &Transaction) {
        let dispute_transaction_opt = fetch_transaction(transaction.id);
        if let Some(dispute_transaction) = dispute_transaction_opt {
            if let Some(amount) = dispute_transaction.amount {
                self.account_engine
                    .update_account(transaction.client_id, |account| {
                        account.available -= amount;
                        account.held += amount;
                    });
            }
        }
    }

    fn handle_resolve(&self, transaction: &Transaction) {
        let dispute_transaction_opt = fetch_transaction(transaction.id);
        if let Some(dispute_transaction) = dispute_transaction_opt {
            if let Some(amount) = dispute_transaction.amount {
                self.account_engine
                    .update_account(transaction.client_id, |account| {
                        account.available += amount;
                        account.held -= amount;
                    });
            }
        }
    }

    fn handle_chargeback(&self, transaction: &Transaction) {
        self.account_engine
            .update_account(transaction.client_id, |account| {
                if let Some(amount) = transaction.amount {
                    account.held -= amount;
                    account.total -= amount;
                    account.locked = true;
                }
            });
    }
}

fn fetch_transaction(transaction_id: u32) -> Option<Transaction> {
    TRANSACTIONS
        .lock()
        .map(|transactions| {
            let trans = transactions.get(&transaction_id);
            trans.cloned()
        })
        .expect("Error locking transactions")
}

fn commit_transaction(transaction: Transaction) {
    TRANSACTIONS
        .lock()
        .map(|mut transactions| {
            transactions.insert(transaction.id, transaction);
        })
        .expect("Error locking transactions");
}
