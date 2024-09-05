# The Little Engine That Could

## Description

This is a small Rust application built to replicate basic transaction processing functionality. Its current
implementation is a simple command line application that reads a CSV file of transactions and outputs the related
accounts after the processing those transactions. Nothing crazy, just a bit of Rust practice!

## Usage

* Clone the repo
* Ensure you have both Rust and Cargo installed
* Run `cargo build` to build the project
* Run `cargo run -- my_csv_location.csv` - Note: this implementation prints the account state to the std out. You can
  redirect the output to a file if you want to save it.

## Testing

* Run `cargo test` to run the test suite

## Architecture

I've tried to keep the architecture pretty simple and grouped the required functionality into different "engines":

* **Ingestion Engine**: This engine is responsible for reading the CSV file in, parsing it and converting to
  valid `Transactions` dependent on the input data.
* **Transaction Engine**: This engine is responsible for consuming the `Transactions` and processing them. This
  currently consists of two steps: updating the underlying account data via the **Account Engine** per transaction type;
  committing the transaction to memory for future reference.
* **Account Engine**: This engine is responsible maintains the state, updating and initialization of accounts based on
  transaction interactions. It also outputs an account summary.
* **Export Engine**: This engine is responsible for outputting account data and currently only supports printing to the
  std out.

## Future Improvements

~~* Engine modularity: This is a current WIP and is available in the `feature/engine_modularity` branch. The idea is to
define and leverage a Trait based approach for the engines to allow for more flexibility and easier testing.~~ Done!

* Move to a queue based ingestion approach.

