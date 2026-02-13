# Multi-Threader

A Rust-based command processor that reads and executes SET, GET, and DELETE commands from an input file.

## Features

- Command parsing with support for SET, GET, and DELETE operations
- In-memory key-value store using HashMap
- Comprehensive error handling and reporting
- Clean modular architecture with separate parser and handler modules

## Project Structure

```
multi_threader/
├── src/
│   ├── main.rs      # Entry point and file processing
│   ├── parser.rs    # Command parsing logic
│   └── handler.rs   # Command execution and data storage
├── input.txt        # Input commands file
└── Cargo.toml       # Project configuration
```

## Supported Commands

### SET
Stores a key-value pair in the data store.
```
SET <key> <value>
```
Example: `SET user:1001 John`

### GET
Retrieves the value associated with a key.
```
GET <key>
```
Example: `GET user:1001`

### DELETE
Removes a key-value pair from the data store.
```
DELETE <key>
```
Example: `DELETE user:1001`

## Usage

1. Create an `input.txt` file with your commands (one command per line)
2. Build and run the project:

```bash
cargo build
cargo run
```

## Example Input File

```
SET user:1001 John
GET user:1001
SET counter:5 100
GET counter:5
DELETE user:1001
GET user:1001
```

## Example Output

```
Starting command processor...
[1] SET user:1001 = John
[2] GET user:1001 = John
[3] SET counter:5 = 100
[4] GET counter:5 = 100
[5] DELETED user:1001 (was: John)
[6] Error: Key 'user:1001' not found

Processed 6 lines
```

## Running Tests

Run the unit tests for the parser and handler modules:

```bash
cargo test
```

## Requirements

- Rust 2024 edition or later
- No external dependencies

## License

This project is for educational purposes.
