use std::collections::HashMap;
use std::sync::Mutex;

use lazy_static::lazy_static;

use crate::models::{Account, ClientId};

lazy_static! {
    pub static ref CLIENT_ACCOUNTS: Mutex<HashMap<ClientId, Account>> = Mutex::new(HashMap::new());
}

pub fn update_account(client_id: ClientId, account_update_func: impl Fn(&mut Account)) {
    CLIENT_ACCOUNTS
        .lock()
        .map(|mut accounts| {
            let account = accounts.entry(client_id).or_insert_with(Account::new);

            // Assumption: An account can't be updated when it's locked. This isn't mentioned in the
            // spec but feels like a reasonable take on expected functionality.
            if !account.locked {
                account_update_func(account);
            }
        })
        .expect("Error locking client accounts");
}

pub fn generate_accounts_summary() -> HashMap<ClientId, Account> {
    CLIENT_ACCOUNTS
        .lock()
        .map(|accounts| accounts.clone())
        .expect("Error locking client accounts")
}

#[cfg(test)]
mod account_tests {
    use sequential_test::sequential;

    use super::*;

    #[test]
    #[sequential]
    fn test_update_account() {
        clear_account_data();

        // given
        let client_id = 1;
        let account_function = |account: &mut Account| {
            account.available += 10.;
            account.total += 10.;
        };

        // when
        update_account(client_id, account_function);

        let account = get_account(client_id);

        // then
        assert_eq!(account.available, 10.);
        assert_eq!(account.total, 10.);
    }

    #[test]
    #[sequential]
    fn test_update_account_sequential() {
        clear_account_data();

        // given
        let client_id = 1;
        let account_function = |account: &mut Account| {
            account.available += 10.;
            account.total += 10.;
        };

        // when
        update_account(client_id, account_function);
        update_account(client_id, account_function);

        let account = get_account(client_id);

        // then
        assert_eq!(account.available, 20.);
        assert_eq!(account.total, 20.);
    }

    #[test]
    #[sequential]
    fn test_update_account_multiple_accounts() {
        clear_account_data();

        // given
        let first_client_id = 1;
        let second_client_id = 2;

        let account_function = |account: &mut Account| {
            account.available += 10.;
            account.total += 10.;
        };

        // when
        update_account(first_client_id, account_function);
        update_account(second_client_id, account_function);

        let first_account = get_account(first_client_id);
        let second_account = get_account(second_client_id);

        // then
        assert_eq!(first_account.available, 10.);
        assert_eq!(second_account.available, 10.);
    }

    #[test]
    #[sequential]
    fn test_update_account_lock() {
        clear_account_data();

        // given
        let client_id = 1;
        let lock_function = |account: &mut Account| {
            account.available += 10.;
            account.locked = true;
        };
        let increment_function = |account: &mut Account| {
            account.available += 10.;
        };

        // when
        update_account(client_id, lock_function);
        update_account(client_id, increment_function);

        let account = get_account(client_id);

        // then
        assert_eq!(account.available, 10.);
    }

    #[test]
    #[sequential]
    fn test_generate_accounts_summary() {
        clear_account_data();

        // given
        let client_id = 1;
        let account_function = |account: &mut Account| {
            account.available += 10.;
            account.total += 10.;
        };

        // when
        update_account(client_id, account_function);
        let accounts_summary = generate_accounts_summary();

        // then
        assert_eq!(accounts_summary.len(), 1);
        assert_eq!(accounts_summary.get(&client_id).unwrap().available, 10.);
        assert_eq!(accounts_summary.get(&client_id).unwrap().total, 10.);
    }

    fn clear_account_data() {
        let mut accounts = CLIENT_ACCOUNTS.lock().unwrap();
        accounts.clear();
    }

    fn get_account(client_id: ClientId) -> Account {
        let accounts = CLIENT_ACCOUNTS.lock().unwrap();
        accounts.get(&client_id).unwrap().clone()
    }
}
