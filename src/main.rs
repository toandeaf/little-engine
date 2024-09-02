use env::args;
use std::env;

use crate::engines::process_transactions_from_csv_file;

mod engines;
mod models;

fn main() {
    let csv_path = args().nth(1).expect("CSV file path not provided");

    process_transactions_from_csv_file(csv_path);
}
