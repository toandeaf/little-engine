use std::env;

use crate::little_engine::process_transactions_from_csv_file;

mod account_engine;
mod export_engine;
mod ingestion_engine;
mod little_engine;
mod models;
mod transaction_engine;

fn main() {
    let csv_path = env::args().nth(1).expect("CSV file path not provided");

    process_transactions_from_csv_file(csv_path);
}
