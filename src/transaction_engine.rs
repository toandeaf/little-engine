use hash_map::Entry;
use std::collections::{hash_map, HashMap};
use std::sync::Mutex;

use lazy_static::lazy_static;

use crate::account_engine::update_account;
use crate::models::{Transaction, TransactionId, TransactionType};

lazy_static! {
    pub static ref TRANSACTIONS: Mutex<HashMap<TransactionId, Transaction>> =
        Mutex::new(HashMap::new());
}

pub fn process_transactions<I>(transactions: I)
where
    I: IntoIterator<Item = Transaction>,
{
    for transaction in transactions {
        match transaction.transaction_type {
            TransactionType::Deposit => handle_deposit(&transaction),
            TransactionType::Withdrawal => handle_withdrawal(&transaction),
            TransactionType::Dispute => handle_dispute(&transaction),
            TransactionType::Resolve => handle_resolve(&transaction),
            TransactionType::Chargeback => handle_chargeback(&transaction),
        }
        match transaction.transaction_type {
            // Assumption: Anything other than deposits or withdrawals are not considered true "transactions"
            // given that their tx ID field references another transaction other than their own.
            TransactionType::Deposit | TransactionType::Withdrawal => {
                commit_transaction(transaction)
            }
            _ => {}
        }
    }
}

fn handle_deposit(transaction: &Transaction) {
    update_account(transaction.client_id, |account| {
        if let Some(amount) = transaction.amount {
            account.available += amount;
            account.total += amount;
        }
    });
}

fn handle_withdrawal(transaction: &Transaction) {
    update_account(transaction.client_id, |account| {
        if let Some(amount) = transaction.amount {
            if account.available < amount {
                return;
            }

            account.available -= amount;
            account.total -= amount;
        }
    });
}

fn handle_dispute(transaction: &Transaction) {
    let tx_to_dispute_opt =
        fetch_and_update_transaction_disputed_state(transaction.id, TransactionType::Dispute);

    if let Some(tx_to_dispute) = tx_to_dispute_opt {
        if let Some(amount) = tx_to_dispute.amount {
            update_account(tx_to_dispute.client_id, |account| {
                account.available -= amount;
                account.held += amount;
            });
        }
    }
}

fn handle_resolve(transaction: &Transaction) {
    let tx_to_resolve_opt =
        fetch_and_update_transaction_disputed_state(transaction.id, TransactionType::Resolve);

    if let Some(tx_to_resolve) = tx_to_resolve_opt {
        if let Some(amount) = tx_to_resolve.amount {
            update_account(tx_to_resolve.client_id, |account| {
                account.available += amount;
                account.held -= amount;
            });
        }
    }
}

fn handle_chargeback(transaction: &Transaction) {
    let tx_to_chargeback_opt =
        fetch_and_update_transaction_disputed_state(transaction.id, TransactionType::Chargeback);

    if let Some(tx_to_chargeback) = tx_to_chargeback_opt {
        if let Some(amount) = tx_to_chargeback.amount {
            update_account(tx_to_chargeback.client_id, |account| {
                account.held -= amount;
                account.total -= amount;
                account.locked = true;
            });
        }
    }
}

fn fetch_and_update_transaction_disputed_state(
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

                    match transaction_type {
                        TransactionType::Dispute => {
                            if transaction.is_disputed {
                                return None;
                            }

                            transaction.is_disputed = true;
                        }
                        TransactionType::Resolve => {
                            if !transaction.is_disputed {
                                return None;
                            }

                            transaction.is_disputed = false;
                        }
                        TransactionType::Chargeback => {
                            if !transaction.is_disputed {
                                return None;
                            }
                        }
                        _ => return None,
                    }

                    Some(transaction.clone())
                }
                Entry::Vacant(_) => None,
            };
        })
        .expect("Error locking transactions")
}

// Assumption: The transaction ID is unique for each entry. This approach doesn't handle duplicates.
fn commit_transaction(transaction: Transaction) {
    TRANSACTIONS
        .lock()
        .map(|mut transactions| {
            transactions.insert(transaction.id, transaction);
        })
        .expect("Error locking transactions");
}

#[cfg(test)]
mod transaction_tests {
    use super::*;

    #[test]
    fn test_single_transaction() {
        clear_transaction_data();

        // given
        let transaction = Transaction {
            id: 1,
            client_id: 1,
            transaction_type: TransactionType::Deposit,
            amount: Some(10.),
            is_disputed: false,
        };

        let transactions = vec![transaction];

        // when
        process_transactions(transactions);

        let process_transactions = get_transactions();

        // then
        assert_eq!(process_transactions.len(), 1);
    }

    #[test]
    fn test_multiple_transactions() {
        clear_transaction_data();

        // given
        let transaction_one = Transaction {
            id: 1,
            client_id: 1,
            transaction_type: TransactionType::Deposit,
            amount: Some(10.),
            is_disputed: false,
        };

        let transaction_two = Transaction {
            id: 2,
            client_id: 1,
            transaction_type: TransactionType::Deposit,
            amount: Some(10.),
            is_disputed: false,
        };

        let transactions = vec![transaction_one, transaction_two];

        // when
        process_transactions(transactions);

        let process_transactions = get_transactions();

        // then
        assert_eq!(process_transactions.len(), 2);
    }

    #[test]
    fn test_dispute_only() {
        clear_transaction_data();

        // given
        let tx = Transaction {
            id: 1,
            client_id: 1,
            transaction_type: TransactionType::Deposit,
            amount: Some(10.),
            is_disputed: false,
        };

        let dispute_tx = Transaction {
            id: 1,
            client_id: 1,
            transaction_type: TransactionType::Dispute,
            amount: None,
            is_disputed: false,
        };

        let transactions = vec![tx, dispute_tx];

        // when
        process_transactions(transactions);

        let process_transactions = get_transactions();

        // then
        assert_eq!(process_transactions.len(), 1);
        assert!(process_transactions[0].is_disputed);
    }

    #[test]
    fn test_dispute_and_resolve() {
        clear_transaction_data();

        // given
        let tx = Transaction {
            id: 1,
            client_id: 1,
            transaction_type: TransactionType::Deposit,
            amount: Some(10.),
            is_disputed: false,
        };

        let dispute_tx = Transaction {
            id: 1,
            client_id: 1,
            transaction_type: TransactionType::Dispute,
            amount: None,
            is_disputed: false,
        };

        let resolve_tx = Transaction {
            id: 1,
            client_id: 1,
            transaction_type: TransactionType::Resolve,
            amount: None,
            is_disputed: false,
        };

        let transactions = vec![tx, dispute_tx, resolve_tx];

        // when
        process_transactions(transactions);

        let process_transactions = get_transactions();

        // then
        assert_eq!(process_transactions.len(), 1);
        assert!(!process_transactions[0].is_disputed);
    }

    fn clear_transaction_data() {
        let mut transactions = TRANSACTIONS.lock().unwrap();
        transactions.clear();
    }

    fn get_transactions() -> Vec<Transaction> {
        let transactions = TRANSACTIONS.lock().unwrap();
        transactions.values().cloned().collect()
    }
}
