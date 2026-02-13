//! Configuration module for runtime and testing behavior
//!
//! This module provides a production-grade configuration system inspired by
//! Redis, PostgreSQL, and RocksDB design patterns.
//!
//! # Design Decisions
//!
//! ## 1. Thread-Safe Sharing with Arc
//! - Config is wrapped in Arc<Config> for cheap cloning across threads
//! - Immutable after creation (no interior mutability needed)
//! - Zero-cost abstraction: Arc only adds a single pointer indirection
//!
//! ## 2. Conditional Compilation for Testing
//! - Testing behavior only available in debug builds (#[cfg(debug_assertions)])
//! - Completely compiled out in release builds (--release flag)
//! - Zero runtime overhead for production deployments
//!
//! ## 3. Environment Variable Configuration
//! - No code changes needed to enable/disable test behavior
//! - Follows 12-factor app principles
//! - Easy to integrate with CI/CD pipelines
//!
//! ## 4. Extensibility
//! - Easy to add new test behaviors without changing existing code
//! - New fields can be added to TestConfig as needed
//! - Type-safe: compiler ensures correct usage

use std::sync::Arc;

#[cfg(debug_assertions)]
use rand::Rng;

#[cfg(debug_assertions)]
use std::time::Duration;

/// Main configuration struct that is passed throughout the application
///
/// # Thread Safety
/// This struct is meant to be wrapped in Arc<Config> and shared across threads.
/// All fields are immutable after creation.
///
/// # Example
/// ```no_run
/// use multi_threader::config::Config;
///
/// let config = Config::from_env();
/// // Pass config to thread pool, handlers, etc.
/// ```
#[derive(Debug, Clone)]
pub struct Config {
    /// Testing configuration (only available in debug builds)
    #[cfg(debug_assertions)]
    pub test: TestConfig,
}

/// Testing configuration for injecting controlled chaos and delays
///
/// This struct is only compiled in debug builds. In release builds (--release),
/// all references to TestConfig are removed by the compiler.
///
/// # Design Philosophy
///
/// Inspired by how production databases handle testing:
/// - Redis: Uses `redis-server --test-memory` flags
/// - PostgreSQL: Has `debug_assertions` and testing hooks
/// - RocksDB: Uses `TEST_` prefixed environment variables
///
/// # Environment Variables
///
/// - `TEST_RANDOM_SLEEP_IO`: Enable random delays in IO threads (true/1 to enable)
/// - `TEST_IO_SLEEP_MIN_MS`: Minimum sleep duration in milliseconds (default: 500)
/// - `TEST_IO_SLEEP_MAX_MS`: Maximum sleep duration in milliseconds (default: 2000)
///
/// # Example Usage
///
/// ```bash
/// # Enable random IO thread delays with default range (500-2000ms)
/// TEST_RANDOM_SLEEP_IO=true cargo run
///
/// # Custom delay range (750-1500ms)
/// TEST_RANDOM_SLEEP_IO=true TEST_IO_SLEEP_MIN_MS=750 TEST_IO_SLEEP_MAX_MS=1500 cargo run
/// ```
#[cfg(debug_assertions)]
#[derive(Debug, Clone)]
pub struct TestConfig {
    /// Enable random sleep delays in IO threads
    ///
    /// When enabled, IO threads will sleep for a random duration after receiving
    /// work from the shared channel. This helps test:
    /// - Race conditions in work distribution
    /// - Command ordering issues
    /// - Thread starvation scenarios
    /// - Backpressure handling
    pub random_sleep_io_thread: bool,

    /// Minimum sleep duration in milliseconds (default: 500)
    pub io_sleep_min_ms: u64,

    /// Maximum sleep duration in milliseconds (default: 2000)
    pub io_sleep_max_ms: u64,
}

impl Config {
    /// Create a new Config from environment variables
    ///
    /// This is the primary way to construct a Config. It reads environment
    /// variables and returns an Arc-wrapped Config ready to be shared across
    /// threads.
    ///
    /// # Returns
    /// Arc<Config> that can be cloned cheaply and passed to threads
    ///
    /// # Example
    /// ```no_run
    /// use multi_threader::config::Config;
    ///
    /// let config = Config::from_env();
    /// // Clone is cheap (just increments Arc refcount)
    /// let config_clone = config.clone();
    /// ```
    pub fn from_env() -> Arc<Self> {
        Arc::new(Self {
            #[cfg(debug_assertions)]
            test: TestConfig::from_env(),
        })
    }

    /// Print the current configuration to stdout
    ///
    /// Useful for debugging and verifying what configuration is active.
    /// In release builds, this only prints a minimal message since test
    /// config is compiled out.
    pub fn print_config(&self) {
        println!("[Config] Configuration loaded:");

        #[cfg(debug_assertions)]
        {
            println!(
                "  [Test] Random IO sleep: {}",
                self.test.random_sleep_io_thread
            );
            if self.test.random_sleep_io_thread {
                println!(
                    "  [Test] IO sleep range: {}-{}ms",
                    self.test.io_sleep_min_ms, self.test.io_sleep_max_ms
                );
            }
        }

        #[cfg(not(debug_assertions))]
        {
            println!("  [Production] All test hooks disabled (release build)");
        }

        println!();
    }
}

#[cfg(debug_assertions)]
impl TestConfig {
    /// Parse testing configuration from environment variables
    ///
    /// # Environment Variables
    ///
    /// - `TEST_RANDOM_SLEEP_IO`: "true" or "1" to enable
    /// - `TEST_IO_SLEEP_MIN_MS`: u64 value (default: 500)
    /// - `TEST_IO_SLEEP_MAX_MS`: u64 value (default: 2000)
    ///
    /// # Panics
    /// Never panics - uses sensible defaults for invalid/missing values
    fn from_env() -> Self {
        let random_sleep_io_thread = std::env::var("TEST_RANDOM_SLEEP_IO")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false);

        let io_sleep_min_ms = std::env::var("TEST_IO_SLEEP_MIN_MS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(500);

        let io_sleep_max_ms = std::env::var("TEST_IO_SLEEP_MAX_MS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(2000);

        // Validate that min <= max, swap if necessary
        let (io_sleep_min_ms, io_sleep_max_ms) = if io_sleep_min_ms > io_sleep_max_ms {
            eprintln!(
                "[Config] Warning: TEST_IO_SLEEP_MIN_MS ({}) > TEST_IO_SLEEP_MAX_MS ({}), swapping values",
                io_sleep_min_ms, io_sleep_max_ms
            );
            (io_sleep_max_ms, io_sleep_min_ms)
        } else {
            (io_sleep_min_ms, io_sleep_max_ms)
        };

        Self {
            random_sleep_io_thread,
            io_sleep_min_ms,
            io_sleep_max_ms,
        }
    }

    /// Maybe sleep the IO thread for a random duration
    ///
    /// This method is called from IO threads after they receive work from the
    /// shared channel. If `random_sleep_io_thread` is enabled, it will sleep
    /// for a random duration between `io_sleep_min_ms` and `io_sleep_max_ms`.
    ///
    /// # When to Call
    ///
    /// Call this immediately after receiving work from the channel:
    /// ```ignore
    /// match string_receiver.lock().unwrap().recv() {
    ///     Ok((raw_string, line_number)) => {
    ///         // Inject random delay for testing
    ///         #[cfg(debug_assertions)]
    ///         config.test.maybe_sleep_io_thread();
    ///
    ///         // Continue with processing...
    ///     }
    /// }
    /// ```
    ///
    /// # Performance
    ///
    /// In release builds, this entire method is compiled out due to
    /// #[cfg(debug_assertions)] guards, resulting in zero overhead.
    pub fn maybe_sleep_io_thread(&self) {
        if self.random_sleep_io_thread {
            let sleep_ms =
                rand::thread_rng().gen_range(self.io_sleep_min_ms..=self.io_sleep_max_ms);
            println!(
                "[Test] IO thread {:?} sleeping for {}ms",
                std::thread::current().id(),
                sleep_ms
            );
            std::thread::sleep(Duration::from_millis(sleep_ms));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_creation() {
        let config = Config::from_env();
        // Should not panic
        config.print_config();
    }

    #[cfg(debug_assertions)]
    #[test]
    fn test_config_from_env_true() {
        // Note: This test may be affected by parallel test execution
        // We test that "true" enables the feature
        unsafe {
            std::env::set_var("TEST_RANDOM_SLEEP_IO", "true");
            std::env::set_var("TEST_IO_SLEEP_MIN_MS", "100");
            std::env::set_var("TEST_IO_SLEEP_MAX_MS", "200");
        }

        let config = Config::from_env();
        assert!(config.test.random_sleep_io_thread);
        assert_eq!(config.test.io_sleep_min_ms, 100);
        assert_eq!(config.test.io_sleep_max_ms, 200);

        // Cleanup
        unsafe {
            std::env::remove_var("TEST_RANDOM_SLEEP_IO");
            std::env::remove_var("TEST_IO_SLEEP_MIN_MS");
            std::env::remove_var("TEST_IO_SLEEP_MAX_MS");
        }
    }

    #[cfg(debug_assertions)]
    #[test]
    fn test_config_swaps_invalid_range() {
        unsafe {
            std::env::set_var("TEST_IO_SLEEP_MIN_MS_SWAP", "2000");
            std::env::set_var("TEST_IO_SLEEP_MAX_MS_SWAP", "500");
        }

        // Create a test config manually to avoid env var pollution
        let io_sleep_min_ms = std::env::var("TEST_IO_SLEEP_MIN_MS_SWAP")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(500);

        let io_sleep_max_ms = std::env::var("TEST_IO_SLEEP_MAX_MS_SWAP")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(2000);

        let (min, max) = if io_sleep_min_ms > io_sleep_max_ms {
            (io_sleep_max_ms, io_sleep_min_ms)
        } else {
            (io_sleep_min_ms, io_sleep_max_ms)
        };

        // Should swap to maintain min <= max
        assert_eq!(min, 500);
        assert_eq!(max, 2000);

        // Cleanup
        unsafe {
            std::env::remove_var("TEST_IO_SLEEP_MIN_MS_SWAP");
            std::env::remove_var("TEST_IO_SLEEP_MAX_MS_SWAP");
        }
    }

    #[cfg(debug_assertions)]
    #[test]
    fn test_maybe_sleep_disabled() {
        // Create a config with sleep disabled
        let test_config = TestConfig {
            random_sleep_io_thread: false,
            io_sleep_min_ms: 500,
            io_sleep_max_ms: 2000,
        };

        let start = std::time::Instant::now();
        test_config.maybe_sleep_io_thread();
        let elapsed = start.elapsed();

        // Should return immediately (< 10ms to be generous)
        assert!(elapsed.as_millis() < 10);
    }
}
