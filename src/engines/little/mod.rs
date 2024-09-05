pub use csv_little::new_little_csv_engine;

use crate::engines::account::AccountEngine;
use crate::engines::export::ExportEngine;
use crate::engines::ingestion::IngestionEngine;
use crate::engines::transaction::TransactionEngine;

mod csv_little;

pub struct LittleEngine<I, T, A, E>
where
    I: IngestionEngine,
    T: TransactionEngine,
    A: AccountEngine,
    E: ExportEngine,
{
    ingestion_engine: I,
    transaction_engine: T,
    account_engine: A,
    export_engine: E,
}

impl<I, T, A, E> LittleEngine<I, T, A, E>
where
    I: IngestionEngine,
    T: TransactionEngine,
    A: AccountEngine,
    E: ExportEngine,
{
    pub(crate) fn process(&self) {
        let transactions = self.ingestion_engine.source_transactions();
        self.transaction_engine.process_transactions(transactions);

        let accounts = self.account_engine.generate_accounts_summary();
        self.export_engine.export_accounts(accounts);
    }
}
