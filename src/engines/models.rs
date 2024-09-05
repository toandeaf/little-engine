#[derive(Clone)]
pub struct Transaction {
    pub(crate) id: u32,
    pub(crate) client_id: ClientId,
    pub(crate) transaction_type: TransactionType,
    pub(crate) amount: Option<f64>,
    pub(crate) is_disputed: bool,
}

#[derive(Clone, PartialEq, Debug)]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Clone)]
pub struct Account {
    pub(crate) available: f64,
    pub(crate) held: f64,
    pub(crate) locked: bool,
}

impl Account {
    pub fn new() -> Self {
        Self {
            available: 0.0,
            held: 0.0,
            locked: false,
        }
    }
}

pub type ClientId = u16;
pub type TransactionId = u32;
