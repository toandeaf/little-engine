use std::io::stdout;

use crate::account_engine::generate_accounts_summary;
use crate::export_engine::export_accounts_as_csv;
use crate::ingestion_engine::parse_transactions_from_csv;
use crate::transaction_engine::process_transactions;

pub fn process_transactions_from_csv_file(csv_path: String) {
    let transactions = parse_transactions_from_csv(csv_path);

    process_transactions(transactions);

    let account_summary = generate_accounts_summary();

    export_accounts_as_csv(account_summary, stdout());
}
