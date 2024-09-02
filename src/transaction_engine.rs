use std::collections::HashMap;
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
        commit_transaction(transaction);
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
            account.available -= amount;
            account.total -= amount;
        }
    });
}

fn handle_dispute(transaction: &Transaction) {
    let dispute_transaction_opt = fetch_transaction(transaction.id);
    if let Some(dispute_transaction) = dispute_transaction_opt {
        if let Some(amount) = dispute_transaction.amount {
            update_account(transaction.client_id, |account| {
                account.available -= amount;
                account.held += amount;
            });
        }
    }
}

fn handle_resolve(transaction: &Transaction) {
    let dispute_transaction_opt = fetch_transaction(transaction.id);
    if let Some(dispute_transaction) = dispute_transaction_opt {
        if let Some(amount) = dispute_transaction.amount {
            update_account(transaction.client_id, |account| {
                account.available += amount;
                account.held -= amount;
            });
        }
    }
}

fn handle_chargeback(transaction: &Transaction) {
    update_account(transaction.client_id, |account| {
        if let Some(amount) = transaction.amount {
            account.held -= amount;
            account.total -= amount;
            account.locked = true;
        }
    });
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

#[cfg(test)]
mod transaction_tests {
    use super::*;

    fn clear_transaction_data() {
        let mut transactions = TRANSACTIONS.lock().unwrap();
        transactions.clear();
    }

    fn get_transactions() -> Vec<Transaction> {
        let transactions = TRANSACTIONS.lock().unwrap();
        transactions.values().cloned().collect()
    }

    #[test]
    fn test_single_transaction() {
        clear_transaction_data();

        // given
        let transaction = Transaction {
            id: 1,
            client_id: 1,
            transaction_type: TransactionType::Deposit,
            amount: Some(10.),
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
        };

        let transaction_two = Transaction {
            id: 2,
            client_id: 1,
            transaction_type: TransactionType::Deposit,
            amount: Some(10.),
        };

        let transactions = vec![transaction_one, transaction_two];

        // when
        process_transactions(transactions);

        let process_transactions = get_transactions();

        // then
        assert_eq!(process_transactions.len(), 2);
    }

    #[test]
    fn test_multiple_transactions_duplicates() {
        clear_transaction_data();

        // given
        let transaction_one = Transaction {
            id: 1,
            client_id: 1,
            transaction_type: TransactionType::Deposit,
            amount: Some(10.),
        };

        let transaction_two = Transaction {
            id: 1,
            client_id: 1,
            transaction_type: TransactionType::Deposit,
            amount: Some(10.),
        };

        let transactions = vec![transaction_one, transaction_two];

        // when
        process_transactions(transactions);

        let process_transactions = get_transactions();

        // then
        assert_eq!(process_transactions.len(), 1);
    }
}
