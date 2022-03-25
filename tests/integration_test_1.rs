use csv;
use floof::{
    client::Client,
    transaction::{RawTransaction, Transaction},
};
use std::collections::HashMap;
use std::convert::TryFrom;
use log::warn;

#[test]
fn integration_test_1() {
    let mut client_map = HashMap::new();
    // let mut tx_map = HashMap::new();

    let mut rdr = csv::Reader::from_path("./tests/test2.csv").unwrap();
    for raw_tx in rdr.deserialize() {
        let raw_tx: RawTransaction = raw_tx.unwrap();
        let tx_entry = Transaction::try_from(raw_tx).unwrap();

        match tx_entry {
            Transaction::Deposit { client, tx, amount } => {
                let client = client_map.entry(client).or_insert(Client::new(client, 0.));
                if !client.add(amount) {
                    warn!("failed to add {} to client {} available balance", amount, client.id());
                }
            }
            // Transaction::Withdrawal { client, tx, amount } => todo!(),
            // Transaction::Dispute { client, tx } => todo!(),
            // Transaction::Resolve { client, tx } => todo!(),
            // Transaction::Chargeback { client, tx } => todo!(),
            _ => (),
        }
    }
}
