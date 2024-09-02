use env::args;
use std::env;

use crate::engines::new_little_csv_engine;

mod engines;

fn main() {
    let csv_path = args().nth(1).expect("CSV file path not provided");

    let little_engine = new_little_csv_engine(csv_path);

    little_engine.process();
}
