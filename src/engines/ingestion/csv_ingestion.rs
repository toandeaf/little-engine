use csv::StringRecord;

use crate::engines::ingestion::IngestionEngine;
use crate::engines::models::{Transaction, TransactionType};

const TYPES_THAT_REQUIRE_AMOUNT: [TransactionType; 2] =
    [TransactionType::Deposit, TransactionType::Withdrawal];

pub struct CSVIngestionEngine {
    csv_file_path: String,
}

impl IngestionEngine for CSVIngestionEngine {
    fn source_transactions(&self) -> Vec<Transaction> {
        let mut csv_file_reader =
            csv::Reader::from_path(&self.csv_file_path).expect("Error reading CSV file");

        let mut transactions = Vec::new();

        for record in csv_file_reader.records().flatten() {
            let transaction_opt = self.create_transaction_from_csv_record(record);
            if let Some(transaction) = transaction_opt {
                transactions.push(transaction);
            }
        }

        transactions
    }
}

impl CSVIngestionEngine {
    pub fn new(csv_file_path: String) -> Self {
        CSVIngestionEngine { csv_file_path }
    }

    fn create_transaction_from_csv_record(&self, record: StringRecord) -> Option<Transaction> {
        let parsed_transaction_type = record.get(0)?;

        let transaction_type = match parsed_transaction_type {
            "deposit" => TransactionType::Deposit,
            "withdrawal" => TransactionType::Withdrawal,
            "dispute" => TransactionType::Dispute,
            "resolve" => TransactionType::Resolve,
            "chargeback" => TransactionType::Chargeback,
            _ => return None,
        };

        self.parse_number_arguments_and_create_transaction(record, transaction_type)
    }

    fn parse_number_arguments_and_create_transaction(
        &self,
        string_record: StringRecord,
        transaction_type: TransactionType,
    ) -> Option<Transaction> {
        let client = self.parse_number_field(&string_record, 1)?;
        let transaction_id = self.parse_number_field(&string_record, 2)?;

        if TYPES_THAT_REQUIRE_AMOUNT.contains(&transaction_type) {
            let amount = self.parse_number_field(&string_record, 3);

            return amount.map(|amount| Transaction {
                transaction_type,
                client_id: client,
                id: transaction_id,
                amount: Some(amount),
                is_disputed: false,
            });
        }

        Some(Transaction {
            transaction_type,
            client_id: client,
            id: transaction_id,
            amount: None,
            is_disputed: false,
        })
    }

    fn parse_number_field<T: std::str::FromStr>(
        &self,
        record: &StringRecord,
        index: usize,
    ) -> Option<T> {
        record.get(index)?.trim().parse().ok()
    }
}
