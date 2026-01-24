# Exam / Ben S 
# Development Environment
`cargo 1.77.1 (e52e36006 2024-03-26)`

Using `cargo fmt` and `cargo clippy` before commit. 

Using VCScode w/GitHub Co-Pilot disabled. 

# Setup / Running
`cargo build`

`cargo run test_data/01_basic_deposits_withdrawals_input.csv `

# Testing

Deposit/Withdrawal:
`cargo run -- test_data/01_basic_deposits_withdrawals_input.csv > output.csv; diff output.csv test_data/01_basic_deposits_withdrawals_expected.csv`

Deposit/Dispute:
`cargo run -- test_data/03_dispute_input.csv > output.csv; diff output.csv test_data/03_dispute_expected.csv`

Deposit/Dipute/Resolve:
`cargo run -- test_data/04_dispute_resolve_input.csv > output.csv; diff output.csv test_data/04_dispute_resolve_expected.csv`

Chargeback (Fails because not implemented):
`cargo run -- test_data/05_chargeback_input.csv > output.csv; diff output.csv test_data/05_chargeback_expected.csv`

Decimal precision: 
`cargo run -- test_data/09_decimal_precision_input.csv > output.csv; diff output.csv test_data/09_decimal_precision_expected.csv `

Test all cases:
`cargo run -- test_data/comprehensive_test_input.csv > output.csv; diff output.csv test_data/comprehensive_test_expected.csv`

# Design
* Streaming will likely be 1) more performant & 2) simpler (less internal state)
* Sychronous processing of events for simplicity/debugging ease

# TODO
* [DONE] Create crate
* [DONE] Parse argument
* [DONE] Define data structures (Account, Transaction)
* [DONE] Stream CSV with serde deserialization
* [DONE] Build core logic (deposit, withdrawal)
* [DONE] Build dispute logic (dispute, resolve)
* [DONE] Implement chargeback
* [DONE] Output results to stdout (Need to fix eprintlns()/stderr maybe or just leave for debugging)
* [DONE] Change println into logging
* [DONE] Confirm all decimals are good to 4 places
* Handle edge cases
* Test against test data
* Clean up nested if's into more idoimatic rust ways
  * Refactor into specific functions
  * [DONE] Use csvwriter? for output

# BUGS
* HashMap ordering non-deterministic - output varies / tests fail
  * This is not a bug because spec says order doesn't matter, it's just that my hacky test harness is using diff so order _does_ matter. Will fix later when moving to unit tests.  

# Assumptions
* I am _not_ hard failing if a bad row comes in from the CSV - if we think in the case of a bank or atm, I think they would raise this internally
* Deposit is the only action that creates an account - therefor the account must exist for any other action to succeed


# Future Work
* Implement more robust arg parsing
* Build out test harness
* Performance profiling

# Resources
* Rust In Action: https://www.manning.com/books/rust-in-action
* CLI args: https://rust-cli.github.io/book/tutorial/cli-args.html
* File Handling
  * https://users.rust-lang.org/t/rust-file-open-error-handling/50681
  * BufRead: https://doc.rust-lang.org/std/io/struct.BufReader.html
* Decimal/float handling crate research: 
  * https://www.reddit.com/r/rust/comments/a7frqj/have_anyone_reviewed_any_of_the_decimal_crates/
  * https://docs.rs/rust_decimal/latest/rust_decimal/
* Transaction type enums: 
  * https://doc.rust-lang.org/reference/items/enumerations.html
  * https://doc.rust-lang.org/book/ch06-02-match.html
* Idioms:
  * https://users.rust-lang.org/t/idiomatic-way-to-set-string-default-value-to-unwrap/29228
    * https://doc.rust-lang.org/rust-by-example/error/option_unwrap.html
    * https://doc.rust-lang.org/rust-by-example/std/result/question_mark.html
* Logging:
  * https://crates.io/crates/log
  * https://crates.io/crates/env_logger
  * https://crates.io/crates/log2
* LLM for syntax error debugging/help/generating text file test cases