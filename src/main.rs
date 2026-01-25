use csv::ReaderBuilder;
use std::fs::File;
use std::io::BufReader;

use rust_decimal::Decimal;
use serde::Deserialize;

use std::collections::HashMap;

use log2::*;

use csv::Writer;
use serde::Serialize;

#[cfg(test)]
mod tests;

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
#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Serialize)]
struct OutputRecord {
    client: u16,
    available: Decimal,
    held: Decimal,
    total: Decimal,
    locked: bool,
}

fn handle_deposit(
    transaction: TransactionRow,
    accounts: &mut HashMap<u16, AccountRecord>,
    transactions: &mut HashMap<u32, TransactionRow>,
) -> Result<(), String> {
    let amount = transaction
        .amount
        .filter(|a| *a > Decimal::ZERO) // Don't allow zero deposit
        .ok_or_else(|| format!("Deposit tx:{} must have a valid amount", transaction.tx))?;

    if transactions.contains_key(&transaction.tx) {
        return Err(format!("Duplicate transaction ID: {}", transaction.tx));
    }

    // Only create the account when there is a valid amount
    // Only persist the account when there is a valid amount
    let account = accounts.entry(transaction.client).or_default();

    // This isn't explicit in the Specification, but was uncovered during testing
    // If the account is locked, we cannot deposit to (or withdraw from) it
    if account.locked {
        return Err(format!("Account: {} is locked", transaction.client));
    }

    account.available += amount;
    transactions.insert(transaction.tx, transaction);

    Ok(())
}

fn handle_withdrawal(
    transaction: &TransactionRow,
    accounts: &mut HashMap<u16, AccountRecord>,
) -> Result<(), String> {
    let amount = transaction
        .amount
        .filter(|a| *a > Decimal::ZERO) // Don't allow zero withdrawal
        .ok_or_else(|| format!("Withdrawal tx:{} must have a valid amount", transaction.tx))?;

    let account = accounts.get_mut(&transaction.client).ok_or_else(|| {
        format!(
            "Account: {} does not exist for withdrawal",
            transaction.client
        )
    })?;

    // Apply the same logic in Deposit for a locked account
    if account.locked {
        return Err(format!("Account: {} is locked", transaction.client));
    }

    if account.available < amount {
        return Err(format!(
            "Insufficient funds: tried to withdraw {} from available {}",
            amount, account.available
        ));
    }

    account.available -= amount;

    Ok(())
}

fn handle_dispute(
    transaction: &TransactionRow,
    accounts: &mut HashMap<u16, AccountRecord>,
    transactions: &mut HashMap<u32, TransactionRow>,
) -> Result<(), String> {
    let disputed_tx = transactions.get_mut(&transaction.tx).ok_or_else(|| {
        format!(
            "Dispute references non-existent transaction: {}",
            transaction.tx
        )
    })?;

    // Found while testing, cannot dispute the same transaction > 1 time
    if disputed_tx.client != transaction.client {
        return Err(format!(
            "Client: {} cannot dispute transaction belonging to client: {}",
            transaction.client, disputed_tx.client
        ));
    }

    if disputed_tx.disputed {
        return Err(format!(
            "Transaction: {} is already under dispute",
            transaction.tx
        ));
    }

    let amount = disputed_tx
        .amount
        .ok_or_else(|| format!("Transaction: {} has no amount", transaction.tx))?;

    let account = accounts
        .get_mut(&transaction.client)
        .ok_or_else(|| format!("Account: {} does not exist", transaction.client))?;

    // Per Specification, "clients available funds should decrease by amount disputed"
    // Per Specification, "held funds should increase by the amount disputed"
    account.available -= amount;
    account.held += amount;
    // We check later if a transaction is under dispute
    disputed_tx.disputed = true;

    Ok(())
}

fn handle_resolve(
    transaction: &TransactionRow,
    accounts: &mut HashMap<u16, AccountRecord>,
    transactions: &mut HashMap<u32, TransactionRow>,
) -> Result<(), String> {
    let resolved_tx = transactions.get_mut(&transaction.tx).ok_or_else(|| {
        format!(
            "Resolve references non-existent transaction: {}",
            transaction.tx
        )
    })?;

    // Verify transaction belongs to this client
    if resolved_tx.client != transaction.client {
        return Err(format!(
            "Client: {} cannot resolve transaction belonging to client: {}",
            transaction.client, resolved_tx.client
        ));
    }

    // Check if transaction is under dispute
    if !resolved_tx.disputed {
        return Err(format!(
            "Transaction: {} is not under dispute",
            transaction.tx
        ));
    }

    let amount = resolved_tx
        .amount
        .ok_or_else(|| format!("Transaction: {} has no amount", transaction.tx))?;

    let account = accounts
        .get_mut(&transaction.client)
        .ok_or_else(|| format!("Account: {} does not exist", transaction.client))?;

    account.held -= amount;
    account.available += amount;
    resolved_tx.disputed = false;

    Ok(())
}

fn handle_chargeback(
    transaction: &TransactionRow,
    accounts: &mut HashMap<u16, AccountRecord>,
    transactions: &mut HashMap<u32, TransactionRow>,
) -> Result<(), String> {
    let chargeback_tx = transactions.get_mut(&transaction.tx).ok_or_else(|| {
        format!(
            "Chargeback references non-existent transaction: {}",
            transaction.tx
        )
    })?;

    // Verify chargeback request belongs to this client
    if chargeback_tx.client != transaction.client {
        return Err(format!(
            "Client: {} cannot chargeback transaction belonging to client: {}",
            transaction.client, chargeback_tx.client
        ));
    }

    // Specification says a 'chargeback is the final state of a dispute'
    // So account must be under 'dispute' to initiate a chargeback
    if !chargeback_tx.disputed {
        return Err(format!(
            "Transaction: {} is not under dispute and cannot be charged back",
            transaction.tx
        ));
    }

    let amount = chargeback_tx
        .amount
        .ok_or_else(|| format!("Transaction: {} has no amount", transaction.tx))?;

    let account = accounts
        .get_mut(&transaction.client)
        .ok_or_else(|| format!("Account: {} does not exist", transaction.client))?;

    account.held -= amount;
    account.locked = true;
    // Found while testing, a chargeback is no longer under dispute
    chargeback_tx.disputed = false;

    Ok(())
}

fn main() {
    let _log2 = log2::open("run_log.txt").start();

    let transaction_csv = std::env::args().nth(1).expect("Transactions csv required");

    // maybe look to use ? | have main() return a result
    let transaction_file = match File::open(transaction_csv) {
        Ok(transaction_file) => transaction_file,
        Err(err) => {
            error!("Invalid transactions file: {}", err);
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
        let result = match transaction.tx_type {
            TransactionType::Deposit => {
                handle_deposit(transaction, &mut all_accounts, &mut all_transactions)
            }
            TransactionType::Withdrawal => handle_withdrawal(&transaction, &mut all_accounts),
            TransactionType::Dispute => {
                handle_dispute(&transaction, &mut all_accounts, &mut all_transactions)
            }
            TransactionType::Resolve => {
                handle_resolve(&transaction, &mut all_accounts, &mut all_transactions)
            }
            TransactionType::Chargeback => {
                handle_chargeback(&transaction, &mut all_accounts, &mut all_transactions)
            }
        };

        if let Err(e) = result {
            error!("Transaction failed: {}", e);
        }
    }

    let mut output_writer = Writer::from_writer(std::io::stdout());
    for (client_id, account) in &all_accounts {
        if let Err(e) = output_writer.serialize(OutputRecord {
            client: *client_id, // OutputRecord does not want a reference
            available: account.available,
            held: account.held,
            total: account.available + account.held,
            locked: account.locked,
        }) {
            error!("Failed to serialize output: {}", e);
        }
    }
    if let Err(e) = output_writer.flush() {
        error!("Failed to flush output: {}", e);
    }
}
