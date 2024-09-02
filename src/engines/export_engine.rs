use std::collections::HashMap;
use std::io::Write;

use serde::Serialize;

use crate::models::{Account, ClientId};

#[derive(Serialize)]
struct AccountCSVRecord {
    client: ClientId,
    available: String,
    held: String,
    total: String,
    locked: bool,
}

pub fn export_accounts_as_csv<W: Write>(accounts: HashMap<ClientId, Account>, writer: W) {
    let mut csv_writer = csv::Writer::from_writer(writer);

    for (client_id, account) in accounts.iter() {
        csv_writer
            .serialize(AccountCSVRecord {
                client: *client_id,
                available: format_float(account.available),
                held: format_float(account.held),
                total: format_float(account.total),
                locked: account.locked,
            })
            .expect("Error serializing account data");
    }

    csv_writer.flush().expect("Error flushing CSV writer");
}

fn format_float(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{:.0}", value)
    } else {
        format!("{:.4}", value)
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::models::Account;

    use super::*;

    #[test]
    fn test_your_function_output() {}

    #[test]
    fn test_format_float() {
        assert_eq!(format_float(0.0), "0");
        assert_eq!(format_float(1.0), "1");
        assert_eq!(format_float(1.2345), "1.2345");
        assert_eq!(format_float(1.23456789), "1.2346");
        assert_eq!(format_float(1.234567890), "1.2346");
    }

    #[test]
    fn test_export_accounts_as_csv() {
        // given
        let mut accounts = HashMap::new();
        accounts.insert(
            1,
            Account {
                available: 1.0,
                held: 2.0,
                total: 3.0,
                locked: false,
            },
        );
        accounts.insert(
            2,
            Account {
                available: 4.55555555,
                held: 5.0,
                total: 9.55555555,
                locked: true,
            },
        );

        let mut test_stdout = Vec::new();

        // when
        export_accounts_as_csv(accounts, &mut test_stdout);

        let test_output_string =
            String::from_utf8(test_stdout).expect("Error converting stdout to string");

        let csv_lines = test_output_string.split("\n").collect::<Vec<&str>>();

        // then
        assert_eq!(csv_lines.len(), 4);
        assert!(csv_lines.contains(&"client,available,held,total,locked"));
        assert!(csv_lines.contains(&"1,1,2,3,false"));
        assert!(csv_lines.contains(&"2,4.5556,5,9.5556,true"));
    }
}
