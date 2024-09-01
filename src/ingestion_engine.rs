use csv::StringRecord;

use crate::models::{Transaction, TransactionType};

pub fn parse_transactions_from_csv(csv_file_path: String) -> Vec<Transaction> {
    let mut csv_file_reader =
        csv::Reader::from_path(csv_file_path).expect("Error reading CSV file");

    let mut transactions = Vec::new();

    for record in csv_file_reader.records().flatten() {
        let transaction_opt = create_transaction_from_csv_record(record);
        if let Some(transaction) = transaction_opt {
            transactions.push(transaction);
        }
    }

    transactions
}

fn create_transaction_from_csv_record(record: StringRecord) -> Option<Transaction> {
    let parsed_transaction_type = record.get(0)?;

    let transaction_type = match parsed_transaction_type {
        "deposit" => TransactionType::Deposit,
        "withdrawal" => TransactionType::Withdrawal,
        "dispute" => TransactionType::Dispute,
        "resolve" => TransactionType::Resolve,
        "chargeback" => TransactionType::Chargeback,
        _ => return None,
    };

    parse_number_arguments_and_create_transaction(record, transaction_type)
}

fn parse_number_arguments_and_create_transaction(
    record: StringRecord,
    record_type: TransactionType,
) -> Option<Transaction> {
    let client = parse_number_field(&record, 1)?;
    let transaction_id = parse_number_field(&record, 2)?;
    let amount = parse_number_field(&record, 3);

    Some(Transaction {
        record_type,
        client_id: client,
        id: transaction_id,
        amount,
    })
}

fn parse_number_field<T: std::str::FromStr>(record: &StringRecord, index: usize) -> Option<T> {
    record.get(index)?.trim().parse().ok()
}
