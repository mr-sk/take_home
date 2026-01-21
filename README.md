# Exam / Ben S 
# Setup / Running
 `cargo run test_data/01_basic_deposits_withdrawals_input.csv `

# Testing
`cargo run -- test_data/01_basic_deposits_withdrawals_input.csv > output.csv; diff output.csv test_data/01_basic_deposits_withdrawals_expected.csv`


# Design
* Streaming will likely be 1) more performant & 2) simpler (less internal state)
* Sychronous processing of events for simplicity/debugging ease

# TODO
* [DONE] Create crate
* [DONE] Parse argument
* [DONE] Define data structures (Account, Transaction)
* [DONE] Stream CSV with serde deserialization
* [DONE] Build core logic (deposit, withdrawal)
* Build dispute logic (dispute, resolve, chargeback)
* [DONE?] Output results to stdout (Need to fix eprintlns()/stderr maybe or just leave for debugging)
* Handle edge cases
* Test against test data
* Clean up nested if's into more idoimatic rust ways

# Assumptions
* I am _not_ hard failing if a bad row comes in from the CSV - if we think in the case of a bank or atm, I think they would raise this internally
* Deposit is the only action that creates an account - therefor the account must exist for any other action to succeed


# Future Work
* Implement more robust arg parsing
* Build out test harness
* Performance profiling