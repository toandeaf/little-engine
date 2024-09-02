use std::collections::HashMap;

use serde::Serialize;

use crate::engines::export::ExportEngine;
use crate::engines::models::{Account, ClientId};

pub struct CSVExportEngine;

#[derive(Serialize)]
struct AccountCSVRecord {
    client: ClientId,
    available: String,
    held: String,
    total: String,
    locked: bool,
}

impl ExportEngine for CSVExportEngine {
    fn export_accounts(&self, accounts: HashMap<ClientId, Account>) {
        let mut csv_writer = csv::Writer::from_writer(std::io::stdout());

        for (client_id, account) in accounts.iter() {
            csv_writer
                .serialize(AccountCSVRecord {
                    client: *client_id,
                    available: self.format_float(account.available),
                    held: self.format_float(account.held),
                    total: self.format_float(account.total),
                    locked: account.locked,
                })
                .expect("Error serializing in_memory_account data");
        }

        csv_writer.flush().expect("Error flushing CSV writer");
    }
}

impl CSVExportEngine {
    pub fn new() -> Self {
        CSVExportEngine
    }

    // TODO rework this
    fn format_float(&self, value: f64) -> String {
        if value.fract() == 0.0 {
            format!("{:.0}", value)
        } else {
            format!("{:.4}", value)
                .trim_end_matches('0')
                .trim_end_matches('.')
                .to_string()
        }
    }
}
