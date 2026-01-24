# Exam / Ben S 
# Development Environment
`cargo 1.77.1 (e52e36006 2024-03-26)`

Using `cargo fmt` and `cargo clippy` before commit. 

Using VCScode w/GitHub Co-Pilot disabled. 

# Setup / Running
Clone the repo then `cargo test`

Run a specific unit test:
* `cargo test -- resolve_moves_funds_back_to_available`

Run a specific integration test:
* `cargo test -- test_01_basic_deposits_withdrawals`

# Manual Testing

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
* [DONE] Handle edge cases
* [DONE] Test against test data
* [DONE] Clean up nested if's into more idoimatic rust ways
  * [DONE] Refactor into specific functions
  * [DONE] Use csvwriter? for output

# LLM Usage
* At this (https://github.com/mr-sk/take_home/commit/1de1e32bd36f91ddaaab3f925662a2773b1f8599) commit, I had Claude generate a full integration testing suite:
  * It took a few iterations/feedback to get it correct/simple
  * Previouly it had generated csv files I used in my development loop
* Now I had it write the tests/integration_tests.rs file. That led me to discover and fix the following bugs:
  * Double dispute could happen
  * Resetting chargeback state once frozen  
* Now I have a working program, with good test coverage. I know the nesting in the match is gross and needs to be refactored. My approach would be to move the majority of the logic into functions that are called under each match. This would streamline that block, encapsulate the logic and allow for unit testing. Unit testing is difficult now because it is one giant function. 
* At this commit (https://github.com/mr-sk/take_home/commit/383393a9279ef77f2f63e62f2e77b2f2415e10e9), I had Claude refactor the script with the above goals expressed. I took the ouput and moved one function at a time, making sure I could follow the logic, the logging was detailed (it had removed all arguments), and comments were correct (it dropped those as well). After each function was ported, I ran the integration test framework. 
* At this commit (https://github.com/mr-sk/take_home/commit/74af650e7440e9f9aef6e4a345430a48026540e0), I had Claude build unit tests, and I broke them into `src/tests.rs`. I now had basic unit tests built, which passed via `cargo test`. I then had Claude generate:
  * Round trip tests: Do an operation and then undo it, validate we are back in start state
  * Invariant tests: Test properties that must remain true
* After those were generated, I ran the full test suite, which succeeded. We now have code that is cleaner, most TODOs are [DONE] and there is both unit and integration tests

# Assumptions
* I am _not_ hard failing if a bad row comes in from the CSV - if we think in the case of a bank or atm, I think they would raise this internally
* Deposit is the only action that creates an account - therefor the account must exist for any other action to succeed

# Future Work
* [DONE] Build out test harness
* Implement more robust arg parsing
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
* Unit Testing: https://doc.rust-lang.org/rust-by-example/testing/unit_testing.html
* LLM for syntax error debugging/help/generating text file test cases