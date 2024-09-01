use std::collections::HashMap;

use serde::Serialize;

use crate::models::{Account, ClientId};

#[derive(Serialize)]
struct AccountCSVRecord {
    client: ClientId,
    available: String,
    held: String,
    total: String,
    locked: bool,
}

pub fn export_accounts_as_csv(accounts: HashMap<ClientId, Account>) {
    let mut csv_writer = csv::Writer::from_writer(std::io::stdout());

    for (client_id, account) in accounts.iter() {
        csv_writer
            .serialize(AccountCSVRecord {
                client: *client_id,
                available: format_float(account.available),
                held: format_float(account.held),
                total: format_float(account.total),
                locked: account.locked,
            })
            .expect("Error serializing account data");
    }

    csv_writer.flush().expect("Error flushing CSV writer");
}

// TODO rework this
fn format_float(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{:.0}", value)
    } else {
        format!("{:.4}", value)
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
    }
}
