use std::{collections::HashMap, convert::TryFrom, io};
use floof::{self, transaction::{RawTransaction, Transaction}, client::Client};
use args::Args;
use structopt::StructOpt;
use log::warn;

pub mod bin_mods;
pub use bin_mods::*;

fn init_logging(verbose: bool) {
    if verbose {
        env_logger::Builder::new()
            .parse_filters("floof=trace")
            .init();
    }
}

fn main() {
    let args = Args::from_args();
    init_logging(args.verbose);

    let mut client_map = HashMap::new();
    let mut tx_map = HashMap::new();

    let mut rdr = csv::Reader::from_path(args.tx_csv).unwrap();
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
                    warn!("{:?}", e);
                } else {
                    if tx_map.contains_key(&tx) {
                        warn!(
                            "another transaction {:?} already exists with \
                            tx id {}. Ignoring this entry",
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
                    warn!("{:?}", e);
                } else {
                    if tx_map.contains_key(&tx) {
                        warn!(
                            "another transaction {:?} already exists with \
                            tx id {}. Ignoring this entry",
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
                        warn!(
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
                                    warn!(
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
                                    warn!(
                                        "tx {} is already being disputed: {:?}",
                                        tx, root_tx
                                    );
                                    continue;
                                }

                                // do not panic on deposit 1, 
                                // withdraw 1, dispute the deposit. Just log it
                                if let Err(e) = client.hold(*amount) {
                                    warn!(
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
                                    warn!(
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
                                    warn!(
                                        "tx {} is already being disputed: {:?}",
                                        tx, root_tx
                                    );
                                    continue;
                                }

                                if let Err(e) = client.add(*amount) {
                                    warn!(
                                        "error adding {}: {:?}",
                                        *amount, e
                                    );
                                } else {
                                    client.hold(*amount).unwrap();
                                }
                            }
                            _ => panic!(
                                "expected root transaction with id {} \
                                to be a deposit or withdrawal type. tx_map
                                bug",
                                tx
                            ),
                        };
                    }
                    None => warn!(
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
                        warn!(
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
                                warn!(
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
                                warn!(
                                    "no dispute to resolve: {:?}", 
                                    tx_entry
                                );
                                continue;
                            }

                            // do not panic on deposit 1, 
                            // withdraw 1, dispute deposit. Just log it
                            if let Err(e) = client.unhold(*amount) {
                                warn!(
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
                                warn!(
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
                                warn!(
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
                                    "error unholding {}: {:?}, \
                                    for client {:?}. Initial dispute bug",
                                    *amount, e, client
                                );
                            } else {
                                if let Err(e) = client.rm(*amount) {
                                    panic!(
                                        "should be no error removing {} \
                                        from client {}: {:?}. Initial dispute \
                                        bug",
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
                            in the tx_map must never be a referring type. \
                            tx_map bug",
                            tx
                        ),
                    },
                    None => warn!("transaction {} does not exist", tx),
                }
            }

            // The client wins. Give them their money directly and lock the
            // compromised account
            Transaction::Chargeback { client, tx } => {
                let client = match client_map.get_mut(&client) {
                    Some(client) => client,
                    None => {
                        warn!(
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
                                    warn!(
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
                                    warn!(
                                        "no dispute to resolve: {:?}",
                                        tx_entry
                                    );
                                    continue;
                                }

                                // deposit 1, withdraw 1, dispute deposit 
                                // success would result in negative balance,
                                // and fail on unhold. Log this
                                if let Err(e) = client.unhold(*amount) {
                                    warn!(
                                        "{:?} should have enough held funds \
                                        from initial dispute to unhold {}: \
                                        {:?}",
                                        client,
                                        *amount,
                                        e
                                    );
                                }

                                if let Err(e) = client.rm(*amount) {
                                    warn!(
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
                                    warn!(
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
                                    warn!(
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
                    None => warn!("transaction {} does not exist", tx),
                }
            }
        }
    }

    let mut wtr = csv::Writer::from_writer(io::stdout());
    for client in client_map.values() {
        wtr.serialize(client).unwrap();
    }
}
