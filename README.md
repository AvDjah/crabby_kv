# Multi-Threader

A high-performance Rust-based multi-threaded command processor that demonstrates concurrent programming patterns including work-stealing, channel-based communication, and thread synchronization using `Arc<Mutex<Receiver>>`.

## Features

- **Work-Stealing Architecture**: IO threads compete for work from a shared channel for optimal load balancing
- **Multi-threaded Processing**: Separate IO threads for parsing and a dedicated thread for command execution
- **Thread-Safe Communication**: Uses `Arc<Mutex<Receiver>>` for multiple consumer pattern
- **Production-Grade Testing Configuration**: Environment-driven chaos testing system inspired by Redis/PostgreSQL
- **Command Support**: SET, GET, and DELETE operations on an in-memory key-value store
- **Graceful Shutdown**: Proper thread lifecycle management with detailed logging
- **Performance Timing**: Detailed timing measurements for each phase of execution
- **Comprehensive Error Handling**: Detailed error reporting with line numbers and thread IDs
- **Clean Modular Architecture**: Separated concerns with parser, handler, thread management, and configuration modules

---

## Table of Contents for LLM Agents

This README is structured to provide complete context for LLM agents. Key sections:

1. **[Architecture](#architecture)** - System design and data flow
2. **[Configuration System](#configuration-system-design-for-llm-agents)** - **START HERE for config changes**
3. **[Project Structure](#project-structure)** - Module organization
4. **[Adding New Features](#how-to-extend-this-project-llm-agent-guide)** - Step-by-step modification guide
5. **[Testing Behaviors](#testing-and-chaos-engineering)** - How to test concurrency issues

---

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
│   ├── config.rs    # Configuration system for runtime and testing behavior
│   ├── thread.rs    # ThreadPool, IoThread, MainThread implementation
│   ├── parser.rs    # Command parsing logic
│   └── handler.rs   # Command execution and data storage
├── input.txt        # Input commands file
└── Cargo.toml       # Project configuration
```

### Module Responsibilities (LLM Reference)

| Module | Purpose | Key Types | Thread Safety |
|--------|---------|-----------|---------------|
| `config.rs` | Runtime configuration, testing hooks | `Config`, `TestConfig` | Immutable `Arc<Config>` shared across threads |
| `thread.rs` | Thread lifecycle, work distribution | `ThreadPool`, `IoThread`, `MainThread` | Uses `Arc<Mutex<Receiver>>` for work-stealing |
| `parser.rs` | String → Command parsing | `Command`, `CommandType` | Stateless, called per-thread |
| `handler.rs` | Command execution, storage | `CommandHandler` | Single-threaded (main thread only) |
| `main.rs` | Entry point, orchestration | N/A | Coordinates all modules |

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

### Basic Usage

1. Create an `input.txt` file with your commands (one command per line)
2. Build and run the project:

```bash
cargo build
cargo run
```

### Testing with Chaos/Delay Injection (Debug Builds Only)

Enable random delays in IO threads to test race conditions and concurrency issues:

```bash
# Enable random IO thread delays (500-2000ms range)
TEST_RANDOM_SLEEP_IO=true cargo run

# Custom delay range (750-1500ms)
TEST_RANDOM_SLEEP_IO=true TEST_IO_SLEEP_MIN_MS=750 TEST_IO_SLEEP_MAX_MS=1500 cargo run

# Test with different thread counts
TEST_RANDOM_SLEEP_IO=true cargo run  # Default: 4 IO threads
```

**Note:** Testing features are only available in debug builds and are completely compiled out in release builds (`cargo run --release`).

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
- Configuration parsing and validation (config.rs)
- End-to-end command processing

---

## Testing and Chaos Engineering

### Available Test Modes

| Mode | Command | Purpose |
|------|---------|---------|
| Normal | `cargo run` | Standard execution |
| Chaos IO | `TEST_RANDOM_SLEEP_IO=true cargo run` | Test work distribution and race conditions |
| Custom Delays | `TEST_IO_SLEEP_MIN_MS=X TEST_IO_SLEEP_MAX_MS=Y` | Control delay range |
| Release | `cargo run --release` | Production mode (all test code removed) |

### What to Test With Random IO Sleep

1. **Work Distribution**
   - Do all IO threads get work, or does one dominate?
   - Does work-stealing balance load properly?

2. **Command Ordering**
   - Are commands processed in a reasonable order?
   - Does the main thread handle out-of-order commands correctly?

3. **Backpressure**
   - What happens when IO threads are slower than the input rate?
   - Does the command channel grow unbounded?

4. **Thread Starvation**
   - Can some threads get starved of work?
   - Does the system recover when threads become fast again?

### Example Test Session

```bash
# Test with aggressive delays
TEST_RANDOM_SLEEP_IO=true TEST_IO_SLEEP_MIN_MS=1000 TEST_IO_SLEEP_MAX_MS=3000 cargo run

# Observe output for:
# - Which threads process which lines
# - Total execution time increase
# - Whether all threads participate
```

---

## Performance Characteristics

- **IO Thread Count**: Configurable (default: 4 threads)
- **Load Balancing**: Automatic via work-stealing
- **Throughput**: Scales with number of IO threads for parsing-heavy workloads
- **Latency**: Sequential command execution ensures consistency

## Requirements

- Rust 2024 edition or later
- Dependencies:
  - `rand = "0.8"` (for testing behavior randomization)

---

## Configuration System Design (FOR LLM AGENTS)

This section provides complete context on the configuration system design for LLM agents that need to understand or modify the testing infrastructure.

### Design Philosophy

The configuration system follows production-grade patterns from mature database systems:

| Project | Pattern Borrowed | Implementation |
|---------|-----------------|----------------|
| **Redis** | `--test-memory` flags, runtime config | Environment variable control |
| **PostgreSQL** | `debug_assertions`, GUC system | Conditional compilation |
| **RocksDB** | `TEST_*` env vars | `TEST_` prefix convention |
| **All DBs** | Thread-safe config sharing | `Arc<Config>` immutable sharing |

### Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│                     Config System                        │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  Environment Variables    →    Config::from_env()       │
│  ↓                                                       │
│  TEST_RANDOM_SLEEP_IO=true  →  Arc<Config>             │
│  TEST_IO_SLEEP_MIN_MS=500                               │
│  TEST_IO_SLEEP_MAX_MS=2000                              │
│                                                          │
│  Arc<Config> passed to:                                 │
│  ├─→ ThreadPool                                         │
│  │   └─→ IoThread (clone Arc)                          │
│  │       └─→ config.test.maybe_sleep_io_thread()       │
│  │                                                       │
│  └─→ Future: MainThread, Handler, etc.                 │
│                                                          │
└─────────────────────────────────────────────────────────┘
```

### Key Design Decisions

#### 1. **Conditional Compilation with `#[cfg(debug_assertions)]`**

**Why:** Zero-cost abstraction - testing code is completely removed in release builds.

**How it works:**
```rust
#[cfg(debug_assertions)]
pub struct TestConfig {
    pub random_sleep_io_thread: bool,
    // ... other fields
}

// In release builds, this entire struct doesn't exist
```

**LLM Agent Note:** When adding new test behaviors, always wrap them in `#[cfg(debug_assertions)]` to maintain zero overhead in production.

#### 2. **Immutable Arc Sharing**

**Why:** Config is read-only after creation, no need for `Arc<Mutex<Config>>` complexity.

**How it works:**
```rust
// In main.rs
let config = Config::from_env();  // Returns Arc<Config>

// In ThreadPool::new()
let mut pool = ThreadPool::new(4, config);  // Arc is moved

// In IoThread::new()
Arc::clone(&config)  // Cheap clone (just increments refcount)
```

**LLM Agent Note:** Never use `Mutex` around `Config` unless you need runtime mutability. The current design is intentionally immutable.

#### 3. **Environment Variable Parsing**

**Why:** No code changes needed, works in CI/CD, follows 12-factor app principles.

**Parsing logic:**
```rust
std::env::var("TEST_RANDOM_SLEEP_IO")
    .map(|v| v == "true" || v == "1")  // Accept "true" or "1"
    .unwrap_or(false)                  // Default to false if not set
```

**LLM Agent Note:** Always provide sensible defaults with `.unwrap_or()`. Never panic on missing env vars.

#### 4. **Injection Point: After Work Reception**

**Where:** In `IoThread::run()`, immediately after `string_receiver.lock().unwrap().recv()` succeeds.

**Why this location:**
- Tests work distribution delays (what if some IO threads are slow?)
- Tests command ordering race conditions
- Tests backpressure on the command channel
- Realistic: simulates slow I/O or network delays

**Code location (src/thread.rs):**
```rust
match result {
    Ok((raw_string, line_number)) => {
        // ← INJECTION POINT: Right after receiving work
        #[cfg(debug_assertions)]
        config.test.maybe_sleep_io_thread();

        // Continue processing...
    }
}
```

### File-by-File Guide for Config Changes

#### **To Add a New Test Behavior:**

**Step 1: Update `src/config.rs`**

```rust
#[cfg(debug_assertions)]
#[derive(Debug, Clone)]
pub struct TestConfig {
    pub random_sleep_io_thread: bool,
    pub io_sleep_min_ms: u64,
    pub io_sleep_max_ms: u64,
    
    // ADD YOUR NEW FIELD HERE
    pub drop_random_commands: bool,  // Example: simulate packet loss
}

#[cfg(debug_assertions)]
impl TestConfig {
    fn from_env() -> Self {
        // ... existing parsing ...
        
        // ADD PARSING FOR YOUR FIELD
        let drop_random_commands = std::env::var("TEST_DROP_COMMANDS")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false);
        
        Self {
            random_sleep_io_thread,
            io_sleep_min_ms,
            io_sleep_max_ms,
            drop_random_commands,  // Include in construction
        }
    }
    
    // ADD YOUR HELPER METHOD
    pub fn maybe_drop_command(&self) -> bool {
        if self.drop_random_commands {
            rand::thread_rng().gen_bool(0.1)  // 10% drop rate
        } else {
            false
        }
    }
}
```

**Step 2: Use it in the relevant module**

Example in `src/thread.rs`:
```rust
match result {
    Ok((raw_string, line_number)) => {
        #[cfg(debug_assertions)]
        if config.test.maybe_drop_command() {
            println!("[Test] Dropping command at line {}", line_number);
            continue;  // Skip processing
        }
        
        // Normal processing...
    }
}
```

**Step 3: Document it in this README**

Add to the "Testing Behaviors" table below.

### Current Testing Behaviors

| Behavior | Env Var | Default | What It Tests |
|----------|---------|---------|---------------|
| **Random IO Sleep** | `TEST_RANDOM_SLEEP_IO=true` | false | Work distribution, race conditions, thread starvation |
| Sleep Min (ms) | `TEST_IO_SLEEP_MIN_MS` | 500 | Delay range control |
| Sleep Max (ms) | `TEST_IO_SLEEP_MAX_MS` | 2000 | Delay range control |

### Future Test Behaviors (Examples for LLM Agents)

Here are examples of additional test behaviors you could add following the same pattern:

1. **Random Command Drops** (`TEST_DROP_COMMANDS`)
   - Simulates packet loss or failed sends
   - Injection: Before `command_sender.send()`

2. **Slow Lock Acquisition** (`TEST_SLOW_LOCKS`)
   - Simulates mutex contention
   - Injection: Add delay before `.lock()`

3. **Main Thread Delays** (`TEST_SLOW_MAIN_THREAD`)
   - Tests backpressure from slow processing
   - Injection: In `MainThread::run()` loop

4. **Parse Errors** (`TEST_RANDOM_PARSE_ERRORS`)
   - Simulates corrupt input
   - Injection: In `parser.rs` before parsing

5. **Memory Pressure** (`TEST_SLOW_ALLOCATIONS`)
   - Simulates low memory conditions
   - Injection: Before `HashMap` operations

### Conditional Compilation Reference

```rust
// Only in debug builds
#[cfg(debug_assertions)]
fn debug_only() { }

// Only in release builds
#[cfg(not(debug_assertions))]
fn release_only() { }

// Only when testing (cargo test)
#[cfg(test)]
fn test_only() { }

// Debug builds OR tests
#[cfg(any(debug_assertions, test))]
fn debug_or_test() { }
```

### Common Pitfalls for LLM Agents

1. **DON'T add `Mutex` around `Config`**
   - Config is immutable, `Arc` alone is sufficient
   - Adding `Mutex` introduces unnecessary locking overhead

2. **DON'T forget `#[cfg(debug_assertions)]`**
   - All test code MUST be conditionally compiled
   - Check with: `cargo build --release` should have zero test overhead

3. **DON'T panic on missing env vars**
   - Always use `.unwrap_or(default_value)`
   - Config should work with zero env vars set

4. **DON'T forget to update tests**
   - Update `src/thread.rs` tests to pass `Config::from_env()`
   - Update any integration tests

5. **DON'T modify config after Arc creation**
   - Config is immutable by design
   - If you need runtime changes, reconsider your approach

### Testing Your Config Changes

```bash
# 1. Verify debug build includes your feature
cargo build
TEST_YOUR_FEATURE=true cargo run

# 2. Verify release build compiles out test code
cargo build --release
# Check binary size - should be same as before

# 3. Run unit tests
cargo test

# 4. Verify env var parsing works
TEST_YOUR_FEATURE=invalid cargo run  # Should use default
TEST_YOUR_FEATURE=true cargo run     # Should enable
```

---

## Concurrency Concepts Demonstrated

This project demonstrates several important Rust concurrency patterns:

1. **Arc (Atomic Reference Counting)**: Safe shared ownership across threads
2. **Mutex**: Mutual exclusion for thread-safe access to the receiver
3. **MPSC Channels**: Message passing between threads
4. **Work-Stealing**: Dynamic load balancing without explicit task assignment
5. **Graceful Shutdown**: Proper thread lifecycle management
6. **Thread Spawning and Joining**: Creating and synchronizing threads
7. **Conditional Compilation**: Zero-cost abstractions for testing

---

## How to Extend This Project (LLM Agent Guide)

This section provides step-by-step instructions for common modifications.

### Adding a New Command Type

**Example:** Add a `COUNT` command that returns the number of keys in the store.

**Step 1: Update `src/parser.rs`**

```rust
#[derive(Debug, PartialEq)]
pub enum CommandType {
    Set(String, String),
    Get(String),
    Delete(String),
    Count,  // ← Add new variant
}

impl FromStr for Command {
    fn from_str(line: &str) -> Result<Self, Self::Err> {
        match parts.as_slice() {
            // ... existing matches ...
            ["COUNT"] => Ok(Command::new(CommandType::Count)),  // ← Add parsing
            _ => Err(format!("Invalid command: {}", trimmed)),
        }
    }
}
```

**Step 2: Update `src/handler.rs`**

```rust
pub fn process_command(&mut self, command: Command) -> Result<String, String> {
    match command.command_type {
        // ... existing matches ...
        CommandType::Count => self.handle_count(),  // ← Add handler
    }
}

fn handle_count(&self) -> Result<String, String> {
    Ok(format!("COUNT = {}", self.store.len()))
}
```

**Step 3: Test your changes**

```bash
echo "COUNT" >> input.txt
cargo run
```

### Adding a New IO Thread Injection Point

**Example:** Add delay before sending to main thread.

**Step 1: Add field to `TestConfig` in `src/config.rs`**

```rust
#[cfg(debug_assertions)]
#[derive(Debug, Clone)]
pub struct TestConfig {
    // ... existing fields ...
    pub random_sleep_before_send: bool,
}
```

**Step 2: Add parsing in `TestConfig::from_env()`**

```rust
let random_sleep_before_send = std::env::var("TEST_SLEEP_BEFORE_SEND")
    .map(|v| v == "true" || v == "1")
    .unwrap_or(false);
```

**Step 3: Add helper method**

```rust
pub fn maybe_sleep_before_send(&self) {
    if self.random_sleep_before_send {
        let sleep_ms = rand::thread_rng().gen_range(100..=500);
        std::thread::sleep(Duration::from_millis(sleep_ms));
    }
}
```

**Step 4: Use in `src/thread.rs`**

```rust
// In IoThread::run(), before command_sender.send()
#[cfg(debug_assertions)]
config.test.maybe_sleep_before_send();

if command_sender.send(msg).is_err() {
    // ...
}
```

### Changing Thread Pool Size

**Option 1: Environment variable (recommended)**

```bash
# Modify main.rs to read env var
NUM_IO_THREADS=8 cargo run
```

**Option 2: Hardcode in `src/main.rs`**

```rust
let num_io_threads = 8;  // Change from 4 to 8
```

### Adding Persistent Storage

**Currently:** Data stored in `HashMap` (in-memory only)

**To add persistence:**

1. Add `serde` dependency to `Cargo.toml`
2. Modify `CommandHandler::new()` to load from file
3. Add `CommandHandler::save()` method
4. Call `.save()` after each command or on shutdown

### Adding a New Thread Type

**Example:** Add a "Logger Thread" that writes all commands to a file.

**Step 1: Create channel in `ThreadPool::new()`**

```rust
let (log_tx, log_rx) = mpsc::channel::<String>();
```

**Step 2: Create `LoggerThread` struct**

```rust
pub struct LoggerThread {
    handle: JoinHandle<()>,
}

impl LoggerThread {
    fn new(log_receiver: Receiver<String>) -> Self {
        let handle = thread::spawn(move || {
            let mut file = File::create("commands.log").unwrap();
            while let Ok(msg) = log_receiver.recv() {
                writeln!(file, "{}", msg).unwrap();
            }
        });
        Self { handle }
    }
}
```

**Step 3: Send to logger from IO threads**

```rust
log_sender.send(format!("Line {}: {}", line_number, raw_string)).ok();
```

---

## License

This project is for educational purposes.
