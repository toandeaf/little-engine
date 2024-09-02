use std::collections::HashMap;
use std::sync::Mutex;

use lazy_static::lazy_static;

use crate::engines::account::AccountEngine;
use crate::engines::models::{Account, ClientId};

lazy_static! {
    pub static ref CLIENT_ACCOUNTS: Mutex<HashMap<ClientId, Account>> = Mutex::new(HashMap::new());
}

#[derive(Clone)]
pub struct InMemoryAccountEngine;

impl AccountEngine for InMemoryAccountEngine {
    fn update_account(&self, client_id: ClientId, account_function: impl Fn(&mut Account)) {
        CLIENT_ACCOUNTS
            .lock()
            .map(|mut accounts| {
                let account = accounts.entry(client_id).or_insert_with(Account::new);

                // TODO - is this necessary? Wasn't mentioned in the specs but feels like it should be
                if !account.locked {
                    account_function(account);
                }
            })
            .expect("Error locking client accounts");
    }

    fn generate_accounts_summary(&self) -> HashMap<ClientId, Account> {
        CLIENT_ACCOUNTS
            .lock()
            .map(|accounts| accounts.clone())
            .expect("Error locking client accounts")
    }
}

impl InMemoryAccountEngine {
    pub fn new() -> Self {
        InMemoryAccountEngine
    }
}
