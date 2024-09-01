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
        match transaction.record_type {
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
