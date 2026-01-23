use csv::ReaderBuilder;
use std::fs::File;
use std::io::BufReader;

use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::Deserialize;

use std::collections::HashMap;

#[derive(Debug, Deserialize)]
struct TransactionRow {
    #[serde(rename = "type")] // 'type' is reserved, fix serde mapping 'type' from .csv
    tx_type: TransactionType,
    client: u16,
    tx: u32,
    amount: Option<Decimal>, // Handles 4 decimal precision and types like dispute
                             // that do not have an 'amount', per the Specification
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
    let transaction_csv = std::env::args()
        .nth(1)
        .expect("[!] transactions csv required");

    let transaction_file = match File::open(transaction_csv) {
        Ok(transaction_file) => transaction_file,
        Err(err) => {
            println!("[!] Invalid transactions file: {}", err);
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
                eprintln!( // TODO: Look at error handling here
                    "[!] Row is being skipped, perhaps should hard fail: {}",
                    err
                );
                continue;
            }
        };
        eprintln!("{:?}", transaction);

        // Check the type of operation this single transaction is
        // TODO: This is sort of gross and sprawling, needs to be streamlined!
        match transaction.tx_type {
            TransactionType::Deposit => {
                // Making an asumption here the account is created during deposit
                // Look up the account or created a 'default' AccountRecord
                let account = all_accounts.entry(transaction.client).or_default();
                eprintln!("account: {:?}", account);

                if transaction.amount.unwrap() < dec!(0) {
                    eprintln!("[!] Deposit cannot be less than 0");
                } else {
                    let tmp_amount = transaction.amount.unwrap();
                    eprintln!("Amount: {}", tmp_amount);
                    account.available += tmp_amount;
                }

                all_transactions.insert(transaction.tx, transaction);
                eprintln!("[debug] {:?}", all_transactions);
            }
            TransactionType::Withdrawal => {
                // The account for the client ID must exist to preform a withdrawal
                if let Some(account) = all_accounts.get_mut(&transaction.client) {
                    eprintln!("account: {:?}", account);

                    if transaction.amount.unwrap() < dec!(0) {
                        eprintln!("[!] Deposit cannot be less than 0");
                    } else {
                        let tmp_amount = transaction.amount.unwrap();
                        eprintln!("Amount: {}", tmp_amount);

                        if account.available >= tmp_amount {
                            account.available -= tmp_amount;

                            eprintln!("Withdrawing {} from account", tmp_amount);
                        } else {
                            eprintln!("Cannot withdrawal more than account has");
                        }
                    }
                } else {
                    eprintln!("Account is required to wirthdraw");
                }
            }
            TransactionType::Dispute => {
                // Fetch the disputed transaction
                if let Some(disputed_transaction) = all_transactions.get_mut(&transaction.tx) {
                    eprintln!("disputed_transaction: {:?}", disputed_transaction);

                    // Fetch the account
                    if let Some(account) = all_accounts.get_mut(&transaction.client) {
                        eprintln!("account: {:?}", account);

                        // Per Specification, "clients available funds should decrease by amount disputed"
                        account.available -= disputed_transaction.amount.unwrap();

                        // Per Specification, "held funds thould increase by the amount disputed"
                        account.held += disputed_transaction.amount.unwrap();
                    }
                } else {
                    eprintln!("[!] Dispute references non-existent transaction: {}", transaction.tx);
                }
            }
            TransactionType::Resolve => {
                  // Fetch the resolved transaction
                  if let Some(resolved_transaction) = all_transactions.get_mut(&transaction.tx) {
                        eprintln!("resolved_transaction: {:?}", resolved_transaction);

                        if let Some(account) = all_accounts.get_mut(&transaction.client) {
                            account.held -= resolved_transaction.amount.unwrap();
                            account.available += resolved_transaction.amount.unwrap();
                        }
                  }else {
                    eprintln!("[!] Resolve references non-existent transaction: {}", transaction.tx);
                }
            }
            _ => {
                eprintln!("[!] Error, unknown type");
            }
        }
    }
    for (client_id, account) in &all_accounts {
        eprintln!("--> Client {}: {:?}", client_id, account);
    }

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
