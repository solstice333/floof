use csv;
use floof::{
    client::Client,
    transaction::{RawTransaction, Transaction},
};
use std::collections::HashMap;
use std::convert::TryFrom;

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

            // Initialize a dispute. Client wants to reverse a withdrawal or 
            // a deposit
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

                if client.is_locked() {
                    continue;
                }

                match tx_map.get_mut(&tx) {
                    Some(root_tx) => {
                        match root_tx {
                            Transaction::Deposit {
                                client: root_id,
                                tx: root_tx,
                                amount,
                                dispute,
                            } => {
                                if *root_id != client.id() {
                                    println!(
                                        "dispute transaction is referring to a \
                                        transaction owned by another client: \
                                        {:?}",
                                        tx_entry
                                    );
                                    continue;
                                }

                                assert_eq!(
                                    *root_tx, tx,
                                    "expected dispute tx id to equal the id \
                                    of the tx being referred to. Transactions \
                                    might be stored in tx_map wrong"
                                );

                                if !*dispute {
                                    *dispute = true;
                                } else {
                                    println!(
                                        "tx {} is already being disputed: {:?}",
                                        tx, root_tx
                                    );
                                    continue;
                                }

                                // do not panic on deposit 1, 
                                // withdraw 1, dispute the deposit. Just log it
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
                                if *root_id != client.id() {
                                    println!(
                                        "dispute transaction is referring to a \
                                        transaction owned by another client: \
                                        {:?}",
                                        tx_entry
                                    );
                                    continue;
                                }

                                assert_eq!(
                                    *root_tx, tx,
                                    "expected dispute tx id to equal the id \
                                    of the tx being referred to. Transactions \
                                    might be stored in tx_map wrong"
                                );

                                if !*dispute {
                                    *dispute = true;
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
                    None => println!(
                        "transaction {} for \
                        client {} does not exist: {:?}", 
                        tx, client.id(), tx_entry
                    ),
                }
            }

            // The client loses. No-op the dispute and return the funds
            // to their former state
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

                if client.is_locked() {
                    continue;
                }

                match tx_map.get_mut(&tx) {
                    Some(root_tx) => match root_tx {
                        Transaction::Deposit {
                            client: root_id,
                            tx: root_tx,
                            amount,
                            dispute,
                        } => {
                            if *root_id != client.id() {
                                println!(
                                    "dispute transaction is referring to a \
                                    transaction owned by another client: \
                                    {:?}",
                                    tx_entry
                                );
                                continue;
                            }

                            assert_eq!(
                                *root_tx, tx,
                                "expected dispute tx id to equal the id \
                                    of the tx being referred to. Transactions \
                                    might be stored in tx_map wrong"
                            );

                            if *dispute {
                                *dispute = false;
                            } else {
                                println!(
                                    "no dispute to resolve: {:?}", 
                                    tx_entry
                                );
                                continue;
                            }

                            // do not panic on deposit 1, 
                            // withdraw 1, dispute deposit. Just log it
                            if let Err(e) = client.unhold(*amount) {
                                println!(
                                    "should be enough held funds in {:?} to \
                                    unhold {}: {:?}",
                                    client, *amount, e
                                );
                            }
                        }
                        Transaction::Withdrawal {
                            client: root_id,
                            tx: root_tx,
                            amount,
                            dispute,
                        } => {
                            if *root_id != client.id() {
                                println!(
                                    "dispute transaction is referring to a \
                                    transaction owned by another client: \
                                    {:?}",
                                    tx_entry
                                );
                                continue;
                            }

                            assert_eq!(
                                *root_tx, tx,
                                "expected dispute tx id to equal the id \
                                    of the tx being referred to. Transactions \
                                    might be stored in tx_map wrong"
                            );

                            if *dispute {
                                *dispute = false;
                            } else {
                                println!(
                                    "no dispute to resolve: {:?}",
                                    tx_entry
                                );
                                continue;
                            }

                            // this must panic b.c. we added funds in to hold
                            // at the initial dispute as a way of saying
                            // "let's pretend the withdrawal never happened
                            // for now until the dispute is settled"
                            if let Err(e) = client.unhold(*amount) {
                                panic!(
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

            // The client wins. Give them their money directly and lock the
            // compromised account
            Transaction::Chargeback { client, tx } => {
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

                if client.is_locked() {
                    continue;
                }

                match tx_map.get_mut(&tx) {
                    Some(root_tx) => {
                        match root_tx {
                            Transaction::Deposit { 
                                client: root_id, 
                                tx: root_tx, 
                                amount, 
                                dispute 
                            } => {
                                if *root_id != client.id() {
                                    println!(
                                        "dispute transaction is referring to a \
                                        transaction owned by another client: \
                                        {:?}",
                                        tx_entry
                                    );
                                    continue;
                                }

                                assert_eq!(
                                    *root_tx, tx,
                                    "expected dispute tx id to equal the id \
                                    of the tx being referred to. Transactions \
                                    might be stored in tx_map wrong"
                                );

                                if *dispute {
                                    *dispute = false;
                                } else {
                                    println!(
                                        "no dispute to resolve: {:?}",
                                        tx_entry
                                    );
                                    continue;
                                }

                                // deposit 1, withdraw 1, dispute deposit 
                                // success would result in negative balance,
                                // and fail on unhold. Log this
                                if let Err(e) = client.unhold(*amount) {
                                    println!(
                                        "{:?} should have enough held funds \
                                        from initial dispute to unhold {}: \
                                        {:?}",
                                        client,
                                        *amount,
                                        e
                                    );
                                }

                                if let Err(e) = client.rm(*amount) {
                                    println!(
                                        "{:?} should have enough funds to undo \
                                        deposit of {}: {:?}", 
                                        client, 
                                        *amount,
                                        e
                                    );
                                }

                                client.lock();
                            }
                            Transaction::Withdrawal { 
                                client: root_id, 
                                tx: root_tx, 
                                amount, 
                                dispute 
                            } => {
                                if *root_id != client.id() {
                                    println!(
                                        "dispute transaction is referring to a \
                                        transaction owned by another client: \
                                        {:?}",
                                        tx_entry
                                    );
                                    continue;
                                }

                                assert_eq!(
                                    *root_tx, tx,
                                    "expected dispute tx id to equal the id \
                                    of the tx being referred to. Transactions \
                                    might be stored in tx_map wrong"
                                );

                                if *dispute {
                                    *dispute = false;
                                } else {
                                    println!(
                                        "no dispute to resolve: {:?}",
                                        tx_entry
                                    );
                                    continue;
                                }

                                // we added funds on initial dispute to hold as
                                // a way of saying "ok, let's pretend this
                                // withdrawal never happened for now until 
                                // dispute is settled". unhold()/rm() should 
                                // never fail
                                if let Err(e) = client.unhold(*amount) {
                                    panic!(
                                        "{:?} should have enough held funds \
                                        from initial dispute to unhold {}: \
                                        {:?}",
                                        client,
                                        *amount,
                                        e
                                    );
                                }

                                if let Err(e) = client.rm(*amount) {
                                    panic!(
                                        "{:?} should have enough funds to undo \
                                        withdrawal of {}: {:?}", 
                                        client, 
                                        *amount,
                                        e
                                    );
                                }

                                client.lock();
                            }
                            _ => panic!(
                                "expected root transaction with id {} \
                                to be a deposit or withdrawal type. Transactions \
                                in the tx_map must never be a referring type",
                                tx
                            ),
                        }
                    },
                    None => println!("transaction {} does not exist", tx),
                }
            }
        }
    }

    let mut wtr = csv::Writer::from_writer(Vec::new());
    let mut clients: Vec<&Client> = client_map.values().collect();
    clients.sort_by_key(|client| client.id());

    for client in clients {
        wtr.serialize(client).unwrap();
    }

    let lines = String::from_utf8(wtr.into_inner().unwrap()).unwrap();
    let lines: Vec<&str> = lines.split("\n").collect();

    for line in lines.iter() {
        println!("{}", line);
    }

    assert_eq!(lines[0], "client,available,held,total,locked");
    assert_eq!(lines[1], "1,1.0,0.0,1.0,false");
    assert_eq!(lines[2], "2,6.0,0.0,6.0,true");
    assert_eq!(lines[3], "3,20.0,0.0,20.0,false");
}
