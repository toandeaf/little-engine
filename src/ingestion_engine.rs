use csv::StringRecord;

use crate::models::{Transaction, TransactionType};

const TYPES_THAT_REQUIRE_AMOUNT: [TransactionType; 2] =
    [TransactionType::Deposit, TransactionType::Withdrawal];

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
    string_record: StringRecord,
    transaction_type: TransactionType,
) -> Option<Transaction> {
    let client = parse_number_field(&string_record, 1)?;
    let transaction_id = parse_number_field(&string_record, 2)?;

    if TYPES_THAT_REQUIRE_AMOUNT.contains(&transaction_type) {
        let amount = parse_number_field(&string_record, 3);

        return amount.map(|amount| Transaction {
            transaction_type,
            client_id: client,
            id: transaction_id,
            amount: Some(amount),
        });
    }

    Some(Transaction {
        transaction_type,
        client_id: client,
        id: transaction_id,
        amount: None,
    })
}

fn parse_number_field<T: std::str::FromStr>(record: &StringRecord, index: usize) -> Option<T> {
    record.get(index)?.trim().parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_transaction_deposit() {
        let record = StringRecord::from(vec!["deposit", "1", "1", "1"]);

        let transaction = create_transaction_from_csv_record(record).unwrap();
        assert_eq!(transaction.transaction_type, TransactionType::Deposit);
        assert_eq!(transaction.id, 1);
        assert_eq!(transaction.client_id, 1);
        assert_eq!(transaction.amount, Some(1.));
    }

    #[test]
    fn test_create_transaction_withdrawal() {
        let record = StringRecord::from(vec!["deposit", "1", "1", "1"]);

        let transaction = create_transaction_from_csv_record(record).unwrap();
        assert_eq!(transaction.transaction_type, TransactionType::Deposit);
        assert_eq!(transaction.id, 1);
        assert_eq!(transaction.client_id, 1);
        assert_eq!(transaction.amount, Some(1.));
    }

    #[test]
    fn test_create_transaction_dispute() {
        let record = StringRecord::from(vec!["dispute", "1", "1"]);

        let transaction = create_transaction_from_csv_record(record).unwrap();
        assert_eq!(transaction.transaction_type, TransactionType::Dispute);
        assert_eq!(transaction.id, 1);
        assert_eq!(transaction.client_id, 1);
        assert_eq!(transaction.amount, None);
    }

    #[test]
    fn test_create_transaction_resolve() {
        let record = StringRecord::from(vec!["resolve", "1", "1"]);

        let transaction = create_transaction_from_csv_record(record).unwrap();
        assert_eq!(transaction.transaction_type, TransactionType::Resolve);
        assert_eq!(transaction.id, 1);
        assert_eq!(transaction.client_id, 1);
        assert_eq!(transaction.amount, None);
    }

    #[test]
    fn test_create_transaction_chargeback() {
        let record = StringRecord::from(vec!["chargeback", "1", "1"]);

        let transaction = create_transaction_from_csv_record(record).unwrap();
        assert_eq!(transaction.transaction_type, TransactionType::Chargeback);
        assert_eq!(transaction.id, 1);
        assert_eq!(transaction.client_id, 1);
        assert_eq!(transaction.amount, None);
    }

    #[test]
    fn test_create_transaction_invalid_transaction_type() {
        let record = StringRecord::from(vec!["invalid", "1", "1", "1"]);

        let transaction = create_transaction_from_csv_record(record);
        assert!(transaction.is_none());
    }

    #[test]
    fn test_create_transaction_invalid_deposit_without_amount() {
        let record = StringRecord::from(vec!["deposit", "1", "1"]);

        let transaction = create_transaction_from_csv_record(record);
        assert!(transaction.is_none());
    }

    #[test]
    fn test_create_transaction_invalid_withdrawal_without_amount() {
        let record = StringRecord::from(vec!["withdrawal", "1", "1"]);

        let transaction = create_transaction_from_csv_record(record);
        assert!(transaction.is_none());
    }

    #[test]
    fn test_create_transaction_invalid_numbers() {
        let record = StringRecord::from(vec!["withdrawal", "rip", "bozo"]);

        let transaction = create_transaction_from_csv_record(record);
        assert!(transaction.is_none());
    }

    #[test]
    fn test_parse_number_field() {
        let record = StringRecord::from(vec!["1", "2", "3", "should_fail"]);
        assert_eq!(parse_number_field::<u32>(&record, 0), Some(1));
        assert_eq!(parse_number_field::<u32>(&record, 1), Some(2));
        assert_eq!(parse_number_field::<u32>(&record, 2), Some(3));
        assert_eq!(parse_number_field::<u32>(&record, 3), None);
    }
}
