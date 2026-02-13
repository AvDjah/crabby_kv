use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

use crate::handler::CommandHandler;
use crate::parser::Command;

/// Message type for communication between IO threads and main thread
#[derive(Debug)]
pub struct CommandMessage {
    pub command: Command,
    pub line_number: usize,
    pub io_thread_id: usize,
}

/// ThreadPool manages multiple IO threads and one main processing thread
pub struct ThreadPool {
    io_threads: Vec<IoThread>,
    main_thread: Option<MainThread>,
    string_sender: Sender<(String, usize)>,
}

impl ThreadPool {
    /// Creates a new ThreadPool with specified number of IO threads
    pub fn new(num_io_threads: usize) -> Self {
        // Channel for IO threads to send parsed commands to main thread
        let (command_tx, command_rx) = mpsc::channel::<CommandMessage>();

        // Create a SINGLE shared channel for distributing raw strings to IO threads
        let (string_tx, string_rx) = mpsc::channel::<(String, usize)>();

        // Wrap the receiver in Arc<Mutex<>> so all IO threads can share it
        let shared_string_rx = Arc::new(Mutex::new(string_rx));

        // Create IO threads - each gets a clone of the Arc'd receiver
        let mut io_threads = Vec::with_capacity(num_io_threads);
        for id in 0..num_io_threads {
            io_threads.push(IoThread::new(
                id,
                command_tx.clone(),
                Arc::clone(&shared_string_rx),
            ));
        }

        // Drop the original command_tx so only IO threads hold senders
        drop(command_tx);

        // Create main thread
        let main_thread = MainThread::new(command_rx);

        Self {
            io_threads,
            main_thread: Some(main_thread),
            string_sender: string_tx,
        }
    }

    /// Get a sender to submit raw string inputs to IO threads (shared channel)
    pub fn get_string_sender(&self) -> Sender<(String, usize)> {
        self.string_sender.clone()
    }

    /// Start the main processing thread
    pub fn start_main_thread(&mut self) -> JoinHandle<()> {
        self.main_thread
            .take()
            .expect("Main thread already started")
            .start()
    }

    /// Shutdown all threads gracefully
    pub fn shutdown(self) {
        println!("[ThreadPool] Initiating graceful shutdown...");

        // Drop the string_sender to signal IO threads that no more input is coming
        drop(self.string_sender);
        println!("[ThreadPool] String sender dropped - signaling IO threads to finish");

        // Collect all the join handles
        let handles: Vec<_> = self
            .io_threads
            .into_iter()
            .map(|thread| {
                let id = thread.id;
                (id, thread.handle)
            })
            .collect();

        // Wait for all IO threads to finish
        let mut successful_shutdowns = 0;
        let total_threads = handles.len();

        for (id, handle) in handles {
            println!("[ThreadPool] Waiting for IO thread {} to finish...", id);
            match handle.join() {
                Ok(_) => {
                    println!("[ThreadPool] IO thread {} finished successfully", id);
                    successful_shutdowns += 1;
                }
                Err(e) => {
                    eprintln!("[ThreadPool] IO thread {} panicked: {:?}", id, e);
                }
            }
        }

        println!(
            "[ThreadPool] All IO threads shut down ({}/{} successful)",
            successful_shutdowns, total_threads
        );
        // At this point, all IO threads have dropped their command_senders
        // The main thread's receiver will get disconnected and exit naturally
    }
}

/// IO Thread responsible for receiving strings and parsing commands
pub struct IoThread {
    id: usize,
    handle: JoinHandle<()>,
}

impl IoThread {
    fn new(
        id: usize,
        command_sender: Sender<CommandMessage>,
        string_receiver: Arc<Mutex<Receiver<(String, usize)>>>,
    ) -> Self {
        let handle = thread::spawn(move || {
            Self::run(id, string_receiver, command_sender);
        });

        Self { id, handle }
    }

    fn run(
        id: usize,
        string_receiver: Arc<Mutex<Receiver<(String, usize)>>>,
        command_sender: Sender<CommandMessage>,
    ) {
        println!("[IO Thread {}] Started", id);

        // Process incoming strings from the shared channel (work-stealing)
        loop {
            let result = string_receiver.lock().unwrap().recv();

            match result {
                Ok((raw_string, line_number)) => {
                    // Skip empty lines
                    if raw_string.trim().is_empty() {
                        continue;
                    }
                    println!(
                        "[IO Thread {}] Processing line {}: {}",
                        id, line_number, raw_string
                    );

                    // Parse the string into a Command
                    match raw_string.parse::<Command>() {
                        Ok(command) => {
                            let msg = CommandMessage {
                                command,
                                line_number,
                                io_thread_id: id,
                            };

                            // Send to main thread for processing
                            if command_sender.send(msg).is_err() {
                                eprintln!("[IO Thread {}] Main thread disconnected", id);
                                break;
                            }
                        }
                        Err(parse_err) => {
                            eprintln!(
                                "[IO Thread {}] Parse Error at line {}: {} (line: '{}')",
                                id, line_number, parse_err, raw_string
                            );
                        }
                    }
                }
                Err(_) => {
                    // Channel disconnected, no more work
                    println!("[IO Thread {}] Channel disconnected", id);
                    break;
                }
            }
        }

        println!("[IO Thread {}] Shutting down", id);
    }
}

/// Main Thread responsible for processing commands and accessing the store
pub struct MainThread {
    command_receiver: Receiver<CommandMessage>,
}

impl MainThread {
    fn new(command_receiver: Receiver<CommandMessage>) -> Self {
        Self { command_receiver }
    }

    /// Start the main processing thread
    pub fn start(self) -> JoinHandle<()> {
        thread::spawn(move || {
            self.run();
        })
    }

    fn run(self) {
        println!("[Main Thread] Started");

        let mut handler = CommandHandler::new();
        let mut processed_count = 0;

        // Process commands from the queue
        while let Ok(msg) = self.command_receiver.recv() {
            processed_count += 1;

            match handler.process_command(msg.command) {
                Ok(response) => {
                    println!(
                        "[Line {} | IO Thread {}] {}",
                        msg.line_number, msg.io_thread_id, response
                    );
                }
                Err(err) => {
                    eprintln!(
                        "[Line {} | IO Thread {}] Error: {}",
                        msg.line_number, msg.io_thread_id, err
                    );
                }
            }
        }

        println!("[Main Thread] Processed {} commands", processed_count);
        println!("[Main Thread] Shutting down");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_thread_pool_creation() {
        let pool = ThreadPool::new(4);
        assert_eq!(pool.io_threads.len(), 4);
        assert!(pool.main_thread.is_some());
    }

    #[test]
    fn test_command_processing() {
        let mut pool = ThreadPool::new(2);

        // Start main thread
        let main_handle = pool.start_main_thread();

        // Get the shared sender
        let sender = pool.get_string_sender();

        // Send some commands
        sender.send(("SET key1 value1".to_string(), 1)).unwrap();
        sender.send(("GET key1".to_string(), 2)).unwrap();
        sender.send(("DELETE key1".to_string(), 3)).unwrap();

        // Drop sender to signal completion
        drop(sender);

        // Give time for processing
        thread::sleep(Duration::from_millis(100));

        // Shutdown
        pool.shutdown();

        // Wait for main thread to finish
        main_handle.join().unwrap();
    }
}
