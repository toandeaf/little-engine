use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::engines::account::AccountEngine;
use crate::engines::models::{Account, ClientId};

#[derive(Clone)]
pub struct InMemoryAccountEngine {
    accounts: Arc<Mutex<HashMap<ClientId, Account>>>,
}

impl AccountEngine for InMemoryAccountEngine {
    fn update_account(&self, client_id: ClientId, account_function: impl Fn(&mut Account)) {
        self.accounts
            .lock()
            .map(|mut accounts| {
                let account = accounts.entry(client_id).or_insert_with(Account::new);

                if !account.locked {
                    account_function(account);
                }
            })
            .expect("Error locking client accounts");
    }

    fn generate_accounts_summary(&self) -> HashMap<ClientId, Account> {
        self.accounts
            .lock()
            .map(|accounts| accounts.clone())
            .expect("Error locking client accounts")
    }
}

impl InMemoryAccountEngine {
    pub fn new() -> Self {
        InMemoryAccountEngine {
            accounts: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}
