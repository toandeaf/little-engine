use crate::engines::account::InMemoryAccountEngine;
use crate::engines::export::CSVExportEngine;
use crate::engines::ingestion::CSVIngestionEngine;
use crate::engines::little::LittleEngine;
use crate::engines::transaction::InMemoryTransactionEngine;

pub fn new_little_csv_engine(
    csv_path: String,
) -> LittleEngine<
    CSVIngestionEngine,
    InMemoryTransactionEngine<InMemoryAccountEngine>,
    InMemoryAccountEngine,
    CSVExportEngine,
> {
    let ingestion_engine = CSVIngestionEngine::new(csv_path);
    let account_engine = InMemoryAccountEngine::new();
    let transaction_engine = InMemoryTransactionEngine::new(account_engine.clone());
    let export_engine = CSVExportEngine::new();

    LittleEngine {
        ingestion_engine,
        transaction_engine,
        account_engine,
        export_engine,
    }
}
