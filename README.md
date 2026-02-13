# Multi-Threader

A high-performance Rust-based multi-threaded command processor that demonstrates concurrent programming patterns including work-stealing, channel-based communication, and thread synchronization using `Arc<Mutex<Receiver>>`.

## Features

- **Work-Stealing Architecture**: IO threads compete for work from a shared channel for optimal load balancing
- **Multi-threaded Processing**: Separate IO threads for parsing and a dedicated thread for command execution
- **Thread-Safe Communication**: Uses `Arc<Mutex<Receiver>>` for multiple consumer pattern
- **Command Support**: SET, GET, and DELETE operations on an in-memory key-value store
- **Graceful Shutdown**: Proper thread lifecycle management with detailed logging
- **Performance Timing**: Detailed timing measurements for each phase of execution
- **Comprehensive Error Handling**: Detailed error reporting with line numbers and thread IDs
- **Clean Modular Architecture**: Separated concerns with parser, handler, and thread management modules

## Architecture

### Thread Model

The application uses a **two-level channel architecture** with work-stealing for optimal performance:

```
┌─────────────┐
│ Main Thread │ (Reads input.txt)
└──────┬──────┘
       │
       │ Single Shared Channel
       │ (String, line_num)
       ↓
┌──────────────────────────────────────┐
│   Arc<Mutex<Receiver<(String, usize)>>> │  ← Shared by all IO threads
└──────┬───────────────────────────────┘
       │
       ├─→ [IO Thread 0] ──┐
       ├─→ [IO Thread 1] ──┤  Parse commands
       ├─→ [IO Thread 2] ──┤  (Work-stealing pattern)
       └─→ [IO Thread 3] ──┘
                │
                │ Command Channel
                │ (CommandMessage)
                ↓
       ┌────────────────────┐
       │ Main Process Thread │
       │  (Command Handler)  │
       └────────────────────┘
```

### Key Design Patterns

1. **Work-Stealing with Arc<Mutex<Receiver>>**
   - Multiple IO threads share a single receiver wrapped in `Arc<Mutex<>>`
   - Threads acquire lock, receive a message, process it, then release the lock
   - Naturally load-balances: faster threads process more work

2. **Multiple Producer, Single Consumer (MPSC)**
   - IO threads send parsed commands to a single processing thread
   - No synchronization needed on the command channel receiver

3. **Graceful Shutdown**
   - Main thread drops sender to signal completion
   - IO threads exit when channel disconnects
   - ThreadPool waits for all threads to finish cleanly

## Project Structure

```
multi_threader/
├── src/
│   ├── main.rs      # Entry point, file reading, and thread coordination
│   ├── thread.rs    # ThreadPool, IoThread, MainThread implementation
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

The output shows detailed logging including which IO thread processed each line, plus performance timing:

```
Starting multi-threaded command processor...

[Timing] Thread pool created in 160.1µs

[IO Thread 0] Started
[IO Thread 1] Started
[IO Thread 2] Started
[IO Thread 3] Started
[Main Thread] Started

Sent 90 lines to IO threads (work-stealing)
[Timing] File reading and distribution took 208.5µs
[Main] All lines sent, closing input channel

[IO Thread 0] Processing line 1: SET user:1001 John
[IO Thread 1] Processing line 2: GET user:1001
[IO Thread 2] Processing line 3: SET counter:5 100
[Line 1 | IO Thread 0] SET user:1001 = John
[Line 2 | IO Thread 1] GET user:1001 = John
[Line 3 | IO Thread 2] SET counter:5 = 100
...
[ThreadPool] Initiating graceful shutdown...
[ThreadPool] All IO threads shut down (4/4 successful)
[Main Thread] Processed 90 commands
[Main Thread] Shutting down
[Main] Main processing thread finished
[Timing] Main thread completion took 246.9µs

=== All processing complete! ===
[Timing] Total execution time: 1.6463ms

--- Timing Breakdown ---
  Pool creation:       145.7µs
  File reading:        208.5µs
  IO thread shutdown:  915.6µs
  Processing wait:     246.9µs
  Total time:          1.6463ms
```

## Running Tests

Run the comprehensive unit tests for all modules:

```bash
cargo test
```

Tests cover:
- Command parsing (parser.rs)
- Command execution (handler.rs)
- Thread pool creation and management (thread.rs)
- End-to-end command processing

## Performance Characteristics

- **IO Thread Count**: Configurable (default: 4 threads)
- **Load Balancing**: Automatic via work-stealing
- **Throughput**: Scales with number of IO threads for parsing-heavy workloads
- **Latency**: Sequential command execution ensures consistency

## Requirements

- Rust 2024 edition or later
- No external dependencies (uses only `std` library)

## Concurrency Concepts Demonstrated

This project demonstrates several important Rust concurrency patterns:

1. **Arc (Atomic Reference Counting)**: Safe shared ownership across threads
2. **Mutex**: Mutual exclusion for thread-safe access to the receiver
3. **MPSC Channels**: Message passing between threads
4. **Work-Stealing**: Dynamic load balancing without explicit task assignment
5. **Graceful Shutdown**: Proper thread lifecycle management
6. **Thread Spawning and Joining**: Creating and synchronizing threads

## License

This project is for educational purposes.
