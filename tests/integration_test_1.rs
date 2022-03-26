use csv;
use floof::{
    client::Client,
    transaction::{DisputeState, RawTransaction, Transaction},
};
use std::convert::TryFrom;
use std::{collections::HashMap, io};

#[test]
fn integration_test_1() {
    let mut client_map = HashMap::new();
    let mut tx_map = HashMap::new();

    let mut rdr = csv::Reader::from_path("./tests/test2.csv").unwrap();
    for raw_tx in rdr.deserialize() {
        let raw_tx: RawTransaction = raw_tx.unwrap();
        let tx_entry = Transaction::try_from(raw_tx).unwrap();

        match tx_entry {
            Transaction::Deposit {
                client,
                tx,
                amount,
                dispute: _,
            } => {
                let client =
                    client_map.entry(client).or_insert(Client::new(client, 0.));

                if let Err(e) = client.add(amount) {
                    println!("{:?}", e);
                } else {
                    if tx_map.contains_key(&tx) {
                        panic!(
                            "another transaction {:?} \
                            already exists with tx id {}",
                            tx_map.get(&tx).unwrap(),
                            tx
                        );
                    } else {
                        tx_map.insert(tx, tx_entry);
                    }
                }
            }
            Transaction::Withdrawal {
                client,
                tx,
                amount,
                dispute: _,
            } => {
                let client =
                    client_map.entry(client).or_insert(Client::new(client, 0.));

                if let Err(e) = client.rm(amount) {
                    println!("{:?}", e);
                } else {
                    if tx_map.contains_key(&tx) {
                        panic!(
                            "another transaction {:?} \
                            already exists with tx id {}",
                            tx_map.get(&tx).unwrap(),
                            tx
                        );
                    } else {
                        tx_map.insert(tx, tx_entry);
                    }
                }
            }
            Transaction::Dispute { client, tx } => {
                let client = match client_map.get_mut(&client) {
                    Some(client) => client,
                    None => {
                        println!(
                            "a new client with no transaction \
                            history cannot have a dispute"
                        );
                        continue;
                    }
                };

                match tx_map.get_mut(&tx) {
                    Some(root_tx) => {
                        match root_tx {
                            Transaction::Deposit {
                                client: root_id,
                                tx: root_tx,
                                amount,
                                dispute,
                            } => {
                                assert_eq!(
                                    *root_id,
                                    client.id(),
                                    "dispute transaction is referring to a \
                                    transaction owned by another client"
                                );
                                assert_eq!(
                                    *root_tx, tx,
                                    "expected dispute tx id to equal the id \
                                    of the tx being referred to. Transactions \
                                    might be stored in tx_map wrong"
                                );

                                if let DisputeState::None = *dispute {
                                    *dispute = DisputeState::Dispute;
                                } else {
                                    println!(
                                        "tx {} is already being disputed: {:?}",
                                        tx, root_tx
                                    );
                                    continue;
                                }

                                if let Err(e) = client.hold(*amount) {
                                    println!(
                                        "error holding {}: {:?}",
                                        *amount, e
                                    );
                                }
                            }
                            Transaction::Withdrawal {
                                client: root_id,
                                tx: root_tx,
                                amount,
                                dispute,
                            } => {
                                assert_eq!(
                                    *root_id,
                                    client.id(),
                                    "dispute transaction is referring to a \
                                    transaction owned by another client"
                                );
                                assert_eq!(
                                    *root_tx, tx,
                                    "expected dispute tx id to equal the id \
                                    of the tx being referred to. Transactions \
                                    might be stored in tx_map wrong"
                                );

                                if let DisputeState::None = *dispute {
                                    *dispute = DisputeState::Dispute;
                                } else {
                                    println!(
                                        "tx {} is already being disputed: {:?}",
                                        tx, root_tx
                                    );
                                    continue;
                                }

                                if let Err(e) = client.add(*amount) {
                                    println!(
                                        "error adding {}: {:?}",
                                        *amount, e
                                    );
                                } else {
                                    client.hold(*amount).unwrap();
                                }
                            }
                            _ => panic!(
                                "expected root transaction with id {} \
                                to be a deposit or withdrawal type",
                                tx
                            ),
                        };
                    }
                    None => println!("transaction {} does not exist", tx),
                }
            }
            Transaction::Resolve { client, tx } => {
                let client = match client_map.get_mut(&client) {
                    Some(client) => client,
                    None => {
                        println!(
                            "a new client with no transaction \
                            history cannot have a dispute"
                        );
                        continue;
                    }
                };

                match tx_map.get_mut(&tx) {
                    Some(root_tx) => match root_tx {
                        Transaction::Deposit {
                            client: root_id,
                            tx: root_tx,
                            amount,
                            dispute,
                        } => {
                            assert_eq!(
                                *root_id,
                                client.id(),
                                "dispute transaction is referring to a \
                                    transaction owned by another client"
                            );
                            assert_eq!(
                                *root_tx, tx,
                                "expected dispute tx id to equal the id \
                                    of the tx being referred to. Transactions \
                                    might be stored in tx_map wrong"
                            );

                            if let DisputeState::Dispute = *dispute {
                                *dispute = DisputeState::Resolve;
                            } else {
                                println!("no dispute to resolve");
                                continue;
                            }

                            if let Err(e) = client.unhold(*amount) {
                                println!(
                                    "error unholding {}: {:?}",
                                    *amount, e
                                );
                            }
                        }
                        Transaction::Withdrawal {
                            client: root_id,
                            tx: root_tx,
                            amount,
                            dispute,
                        } => {
                            assert_eq!(
                                *root_id,
                                client.id(),
                                "dispute transaction is referring to a \
                                    transaction owned by another client"
                            );
                            assert_eq!(
                                *root_tx, tx,
                                "expected dispute tx id to equal the id \
                                    of the tx being referred to. Transactions \
                                    might be stored in tx_map wrong"
                            );

                            if let DisputeState::Dispute = *dispute {
                                *dispute = DisputeState::Resolve;
                            } else {
                                println!("no dispute to resolve");
                                continue;
                            }

                            if let Err(e) = client.unhold(*amount) {
                                println!(
                                    "error unholding {}: {:?}, for client {:?}",
                                    *amount, e, client
                                );
                            } else {
                                if let Err(e) = client.rm(*amount) {
                                    panic!(
                                        "should be no error removing {} \
                                        from client {}: {:?}, b.c. we added \
                                        funds on initial dispute",
                                        *amount,
                                        client.id(),
                                        e
                                    );
                                }
                            }
                        }
                        _ => panic!(
                            "expected root transaction with id {} \
                            to be a deposit or withdrawal type. Transactions \
                            in the tx_map must never be a referring type",
                            tx
                        ),
                    },
                    None => println!("transaction {} does not exist", tx),
                }
            }
            // Transaction::Chargeback { client, tx } => todo!(),
            _ => (),
        }
    }

    let mut wtr = csv::Writer::from_writer(Vec::new());
    for client in client_map.values() {
        wtr.serialize(client).unwrap();
    }
    println!("{:?}", String::from_utf8(wtr.into_inner().unwrap()));
}
