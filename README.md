![main workflow](https://github.com/jproyo/ans104-indexer/actions/workflows/rust.yml/badge.svg)

# ANS-104 Indexer

This project is an ANS-104 indexer that allows you to index transactions from the ANS-104 bundle using the Arweave network. It provides a command-line interface (CLI) for users to specify transaction IDs and storage options.

## Features

- Index ANS-104 bundle transactions.
- Specify output storage folder.
- Customizable Arweave URL.

## Prerequisites

- Rust (version 1.50 or later)
- Cargo (Rust package manager)

## Installation

1. Clone the repository:
   ```bash
   git clone jproyo/ans104-indexer
   cd ans104-indexer
   ```

2. Build the project:
   ```bash
   cargo build --release
   ```

## Usage

To run the indexer, use the following command:

```bash
cargo run -- --transaction-id <TRANSACTION_ID> [--storage-folder <STORAGE_FOLDER>] [--arwaeve-url <ARWEAVE_URL>] [--help]
```

- `--storage-folder`: Optional. Default is `./storage`.
- `--arwaeve-url`: Optional. Default is `https://arweave.net`.
- `--help`: Displays help information about the command.

### Example

```bash
cargo run -- --transaction-id 123456
```

## Running Tests

To run the tests for the project, use the following command:

```bash
cargo test
```

### Example

```bash
cargo test -- --test-threads=1
```

## License

This project is licensed under the MIT License.

## Future Work

1. Read transaction from storage when it is present instead of downloading it.
2. Adding more tests to improve coverage.
3. Adding streaming support for base64.
