use csv::ReaderBuilder;
use std::fs::File;
use std::io::BufReader;

use rust_decimal::Decimal;
// use rust_decimal_macros::dec;
use serde::Deserialize;

use std::collections::HashMap;

use log2::*;

#[derive(Debug, Deserialize)]
struct TransactionRow {
    #[serde(rename = "type")] // 'type' is reserved, fix serde mapping 'type' from .csv
    tx_type: TransactionType,
    client: u16,
    tx: u32,
    amount: Option<Decimal>, // Handles 4 decimal precision and types like dispute
    // that do not have an 'amount', per the Specification
    #[serde(skip)] // 'disputed' is not in the source CSV
    disputed: bool,
}

#[derive(Debug, Default)]
struct AccountRecord {
    available: Decimal,
    held: Decimal,
    locked: bool,
}

// These are the only transaction types currently supported
#[derive(Debug, Deserialize, PartialEq)] // TODO: Possibly remove PartialEq
#[serde(rename_all = "lowercase")]
enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

fn main() {
    let _log2 = log2::open("run_log.txt").start();

    let transaction_csv = std::env::args()
        .nth(1)
        .expect("[!] transactions csv required");

    // maybe look to use ? | have main() return a result
    let transaction_file = match File::open(transaction_csv) {
        Ok(transaction_file) => transaction_file,
        Err(err) => {
            error!("[!] Invalid transactions file: {}", err);
            std::process::exit(1);
        }
    };

    let file_reader = BufReader::new(transaction_file);
    let mut transaction_csv_reader = ReaderBuilder::new()
        .trim(csv::Trim::All) // Handle whitespace per Specification
        .flexible(true) // Handle non-required fields per Specification
        .from_reader(file_reader);

    // These HashMaps will store all our Account & Transactions Entries by client ID
    let mut all_accounts: HashMap<u16, AccountRecord> = HashMap::new();
    let mut all_transactions: HashMap<u32, TransactionRow> = HashMap::new();

    // Process each row at a time, minimizing memory consumption
    for row in transaction_csv_reader.deserialize::<TransactionRow>() {
        let transaction: TransactionRow = match row {
            Ok(transaction) => transaction,
            Err(err) => {
                warn!("Row is being skipped, error: {}", err);
                continue;
            }
        };

        debug!("Processing Transaction Row: {:?}", transaction);

        // Check the type of operation this single transaction is
        // TODO: This is sort of gross and sprawling, needs to be streamlined!
        match transaction.tx_type {
            TransactionType::Deposit => {
                match transaction.amount {
                    Some(amount) if amount >= Decimal::ZERO => {
                        // Only create the account when there is a valid amount
                        // Only persist the account when there is a valid amount
                        let account = all_accounts.entry(transaction.client).or_default();

                        debug!("Deposit: {:?}", account);

                        account.available += amount;
                        all_transactions.insert(transaction.tx, transaction);

                        debug!("all_transactions after Deposit: {:?}", all_transactions);
                    }
                    Some(amount) => {
                        error!("Deposit cannot be less than 0: {}", amount);
                    }
                    None => {
                        error!("Deposit must have an amount");
                    }
                }
            }
            TransactionType::Withdrawal => {
                if let Some(account) = all_accounts.get_mut(&transaction.client) {
                    match transaction.amount {
                        Some(amount) if amount >= Decimal::ZERO => {
                            if account.available >= amount {
                                account.available -= amount;
                                debug!(
                                    "Withdrew: {} from account, available: {}",
                                    amount, account.available
                                );
                            } else {
                                error!(
                                    "Tried to withdraw: {} from available: {}",
                                    amount, account.available
                                );
                            }
                        }
                        Some(amount) => {
                            error!("Withdrawal cannot be less than 0: {}", amount);
                        }
                        None => {
                            error!("Withdrawal must have an amount");
                        }
                    }
                } else {
                    error!("Account does not exist for withdrawal");
                }
            }
            TransactionType::Dispute => {
                if let Some(disputed_transaction) = all_transactions.get_mut(&transaction.tx) {
                    // Verify transaction belongs to this client
                    if disputed_transaction.client != transaction.client {
                        error!(
                            "Client: {} cannot dispute transaction belonging to client: {}",
                            transaction.client, disputed_transaction.client
                        );
                        continue;
                    }

                    if let Some(amount) = disputed_transaction.amount {
                        if let Some(account) = all_accounts.get_mut(&transaction.client) {
                            // Per Specification, "clients available funds should decrease by amount disputed"
                            // Per Specification, "held funds thould increase by the amount disputed"
                            account.available -= amount;
                            account.held += amount;
                            disputed_transaction.disputed = true; // We later check if a transaction is under dispute
                        }
                    }
                } else {
                    error!(
                        "Dispute references non-existent transaction: {}",
                        transaction.tx
                    );
                }
            }
            TransactionType::Resolve => {
                if let Some(resolved_transaction) = all_transactions.get_mut(&transaction.tx) {
                    // Verify transaction belongs to this client
                    if resolved_transaction.client != transaction.client {
                        error!(
                            "Client: {} cannot resolve transaction belonging to client: {}",
                            transaction.client, resolved_transaction.client
                        );
                        continue;
                    }

                    // Check if transaction is under dispute
                    if !resolved_transaction.disputed {
                        error!("Transaction: {} is not under dispute", transaction.tx);
                        continue;
                    }

                    if let Some(amount) = resolved_transaction.amount {
                        if let Some(account) = all_accounts.get_mut(&transaction.client) {
                            account.held -= amount;
                            account.available += amount;
                            resolved_transaction.disputed = false;
                        }
                    }
                } else {
                    error!(
                        "Resolve references non-existent transaction: {}",
                        transaction.tx
                    );
                }
            }
            _ => {
                error!("Error, unknown type: {:?}", transaction.tx_type);
            }
        }
    }
    for (client_id, account) in &all_accounts {
        debug!("--> Client {}: {:?}", client_id, account);
    }

    // Make into struct that represents the output format
    println!("client,available,held,total,locked");
    for (client_id, account) in &all_accounts {
        println!(
            "{},{},{},{},{}",
            client_id,
            account.available,
            account.held,
            account.available + account.held,
            account.locked
        );
    }
}
