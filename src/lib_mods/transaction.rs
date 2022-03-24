use serde::Deserialize;
use std::{convert::TryFrom, result};

#[cfg(test)]
mod tests {
    use super::RawTransaction;
    use crate::transaction::Transaction;
    use float_cmp::approx_eq;
    use std::convert::TryFrom;

    #[test]
    fn test_raw_tx() {
        let mut rdr = csv::Reader::from_path("./tests/test1.csv").unwrap();
        for rtx in rdr.deserialize() {
            let rtx: RawTransaction = rtx.unwrap();
            assert_eq!(rtx.ty(), "deposit");
            assert_eq!(rtx.client, 1);
            assert_eq!(rtx.tx, 1);
            assert!(approx_eq!(f64, rtx.amount, 1.1234, ulps = 1));
        }
    }

    #[test]
    fn test_convert_raw_tx_to_tx() {
        let rtx = RawTransaction {
            ty: String::from("deposit"),
            client: 1,
            tx: 1,
            amount: 1.2345,
        };
        let rtx = Transaction::try_from(rtx).unwrap();
        match rtx {
            Transaction::Deposit { client, tx, amount } => {
                assert_eq!(client, 1);
                assert_eq!(tx, 1);
                assert!(approx_eq!(f64, amount, 1.2345, ulps = 1));
            },
            _ => panic!("failed to be a deposit transaction")
        }
    }
}

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid transaction: {0}")]
    InvalidTx(String),
}

#[derive(Debug, Deserialize)]
pub struct RawTransaction {
    #[serde(rename = "type")]
    ty: String,
    pub client: u16,
    pub tx: u32,
    pub amount: f64,
}

impl RawTransaction {
    pub fn ty(&self) -> String {
        self.ty.to_ascii_lowercase()
    }
}

#[derive(Debug)]
pub enum Transaction {
    Deposit { client: u16, tx: u32, amount: f64 },
    Withdrawal { client: u16, tx: u32, amount: f64 },
    Dispute { client: u16, tx: u32 },
    Resolve { client: u16, tx: u32 },
    Chargeback { client: u16, tx: u32 },
}

impl TryFrom<RawTransaction> for Transaction {
    type Error = Error;

    fn try_from(rtx: RawTransaction) -> Result<Self> {
        if rtx.ty() == "deposit" {
            Ok(Transaction::Deposit {
                client: rtx.client,
                tx: rtx.tx,
                amount: rtx.amount,
            })
        } else if rtx.ty() == "withdrawal" {
            Ok(Transaction::Withdrawal {
                client: rtx.client,
                tx: rtx.tx,
                amount: rtx.amount,
            })
        } else if rtx.ty() == "dispute" {
            Ok(Transaction::Dispute {
                client: rtx.client,
                tx: rtx.tx,
            })
        } else if rtx.ty() == "resolve" {
            Ok(Transaction::Resolve {
                client: rtx.client,
                tx: rtx.tx,
            })
        } else if rtx.ty() == "chargeback" {
            Ok(Transaction::Chargeback {
                client: rtx.client,
                tx: rtx.tx,
            })
        } else {
            Err(Error::InvalidTx(rtx.ty()))
        }
    }
}
