use std::collections::HashMap;

pub use in_memory_account::InMemoryAccountEngine;

use crate::engines::models::{Account, ClientId};

mod in_memory_account;

pub trait AccountEngine {
    fn update_account(&self, client_id: ClientId, update_fn: impl Fn(&mut Account));
    fn generate_accounts_summary(&self) -> HashMap<ClientId, Account>;
}
