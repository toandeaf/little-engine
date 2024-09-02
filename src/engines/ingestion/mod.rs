pub use csv_ingestion::CSVIngestionEngine;

use crate::engines::models::Transaction;

mod csv_ingestion;

pub trait IngestionEngine {
    fn source_transactions(&self) -> Vec<Transaction>;
}
