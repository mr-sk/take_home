# Exam / Ben S 
# Setup / Running
`cargo build`

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
* Confirm all decimals are good to 4 places
* Handle edge cases
* Test against test data
* Clean up nested if's into more idoimatic rust ways
  * Use csvwriter? for output
  
# BUGS
* HashMap ordering non-deterministic - output varies / tests fail
* 

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
* Open File handling: https://users.rust-lang.org/t/rust-file-open-error-handling/50681
* BufRead: https://doc.rust-lang.org/std/io/struct.BufReader.html
* Decimal/float handling crate research: 
  * https://www.reddit.com/r/rust/comments/a7frqj/have_anyone_reviewed_any_of_the_decimal_crates/
  * https://docs.rs/rust_decimal/latest/rust_decimal/
* Transaction type enums: 
  * https://doc.rust-lang.org/reference/items/enumerations.html
  * https://doc.rust-lang.org/book/ch06-02-match.html
* LLM for syntax error debugging/help