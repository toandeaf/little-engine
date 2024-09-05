use std::collections::HashMap;
use std::sync::Mutex;

use crate::engines::account::AccountEngine;
use crate::engines::models::{Transaction, TransactionId, TransactionType};
use crate::engines::transaction::TransactionEngine;

pub struct InMemoryTransactionEngine<A> {
    account_engine: A,
    transactions: Mutex<HashMap<TransactionId, Transaction>>,
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
        InMemoryTransactionEngine {
            account_engine,
            transactions: Mutex::new(HashMap::new()),
        }
    }

    fn handle_deposit(&self, transaction: &Transaction) {
        self.account_engine
            .update_account(transaction.client_id, |account| {
                if let Some(amount) = transaction.amount {
                    account.available += amount;
                }
            });
    }

    fn handle_withdrawal(&self, transaction: &Transaction) {
        self.account_engine
            .update_account(transaction.client_id, |account| {
                if let Some(amount) = transaction.amount {
                    if amount > account.available {
                        return;
                    }

                    account.available -= amount;
                }
            });
    }

    fn handle_dispute(&self, transaction: &Transaction) {
        let tx_opt = self
            .fetch_and_update_transaction_disputed_state(transaction.id, TransactionType::Dispute);

        tx_opt.and_then(|tx| {
            tx.amount.map(|amount| {
                self.account_engine.update_account(tx.client_id, |account| {
                    if tx.transaction_type == TransactionType::Deposit {
                        account.available -= amount;
                    }

                    account.held += amount;
                });
            })
        });
    }

    fn handle_resolve(&self, transaction: &Transaction) {
        let tx_opt = self
            .fetch_and_update_transaction_disputed_state(transaction.id, TransactionType::Resolve);

        tx_opt.and_then(|tx| {
            tx.amount.map(|amount| {
                self.account_engine.update_account(tx.client_id, |account| {
                    account.available += amount;
                    account.held -= amount;
                });
            })
        });
    }

    fn handle_chargeback(&self, transaction: &Transaction) {
        let tx_opt = self.fetch_and_update_transaction_disputed_state(
            transaction.id,
            TransactionType::Chargeback,
        );

        tx_opt.and_then(|tx| {
            tx.amount.map(|amount| {
                self.account_engine.update_account(tx.client_id, |account| {
                    match tx.transaction_type {
                        TransactionType::Deposit => {
                            eprintln!("Chargeback: {}", amount);
                            // account.available -= amount;
                        }
                        TransactionType::Withdrawal => {
                            account.available += amount;
                        }
                        _ => {}
                    }

                    account.held -= amount;
                    account.locked = true;
                });
            })
        });
    }

    fn fetch_and_update_transaction_disputed_state(
        &self,
        transaction_id: u32,
        transaction_type: TransactionType,
    ) -> Option<Transaction> {
        self.transactions
            .lock()
            .map(|mut transactions| {
                let tx_opt = transactions.get_mut(&transaction_id);

                if let Some(tx) = tx_opt {
                    let transaction_updated =
                        self.update_if_transaction_appropriate(transaction_type, tx);

                    if transaction_updated {
                        return Some(tx.clone());
                    }
                }

                None
            })
            .expect("Error locking transactions")
    }

    // TODO - I can definitely trim this down.
    fn update_if_transaction_appropriate(
        &self,
        transaction_type: TransactionType,
        transaction: &mut Transaction,
    ) -> bool {
        let mut is_valid = false;

        match transaction_type {
            TransactionType::Dispute => {
                if !transaction.is_disputed {
                    transaction.is_disputed = true;
                    is_valid = true;
                }
            }
            TransactionType::Resolve => {
                if transaction.is_disputed {
                    transaction.is_disputed = false;
                    is_valid = true;
                }
            }
            TransactionType::Chargeback => {
                if transaction.is_disputed {
                    is_valid = true;
                }
            }
            _ => {}
        };

        is_valid
    }

    fn commit_transaction(&self, transaction: Transaction) {
        self.transactions
            .lock()
            .map(|mut transactions| {
                if transactions.contains_key(&transaction.id) {
                    return;
                }

                transactions.insert(transaction.id, transaction);
            })
            .expect("Error locking transactions");
    }
}
