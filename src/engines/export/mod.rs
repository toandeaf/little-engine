use std::collections::HashMap;

pub use csv_export::CSVExportEngine;

use crate::engines::models::{Account, ClientId};

mod csv_export;

pub trait ExportEngine {
    fn export_accounts(&self, accounts: HashMap<ClientId, Account>);
}
