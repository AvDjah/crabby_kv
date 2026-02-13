mod handler;
mod parser;
mod thread;

use std::{
    fs::File,
    io::{BufRead, BufReader},
    time::Instant,
};

fn main() {
    let start_time = Instant::now();
    println!("Starting multi-threaded command processor...\n");

    // Create thread pool with 4 IO threads
    let num_io_threads = 4;
    let pool_start = Instant::now();
    let mut pool = thread::ThreadPool::new(num_io_threads);
    let pool_creation_time = pool_start.elapsed();
    println!("[Timing] Thread pool created in {:?}\n", pool_creation_time);

    // Get the single shared sender for all IO threads
    let sender = pool.get_string_sender();

    // Start the main processing thread
    let main_handle = pool.start_main_thread();

    // Open and read the input file
    let file_read_start = Instant::now();
    let f = File::open("input.txt");
    let file_read_time;
    match f {
        Ok(file) => {
            let reader = BufReader::new(file);
            let mut line_num = 0;

            // Send all lines to the shared channel - IO threads will compete for work
            for line_result in reader.lines() {
                match line_result {
                    Ok(line) => {
                        line_num += 1;

                        // Send raw string to shared channel (work-stealing pattern)
                        if let Err(e) = sender.send((line, line_num)) {
                            eprintln!("Failed to send line {} to IO threads: {}", line_num, e);
                            break;
                        }
                    }
                    Err(err) => {
                        eprintln!("Error reading line: {}", err);
                        break;
                    }
                }
            }

            file_read_time = file_read_start.elapsed();
            println!("\nSent {} lines to IO threads (work-stealing)", line_num);
            println!(
                "[Timing] File reading and distribution took {:?}",
                file_read_time
            );
        }
        Err(err) => {
            panic!("Error opening input file: {}", err);
        }
    }

    // Drop sender to signal IO threads that no more input is coming
    drop(sender);
    println!("[Main] All lines sent, closing input channel\n");

    // Shutdown thread pool (this joins all IO threads after they receive disconnect signal)
    let shutdown_start = Instant::now();
    pool.shutdown();
    let shutdown_time = shutdown_start.elapsed();
    println!("[Timing] IO thread shutdown took {:?}\n", shutdown_time);

    // Wait for main thread to finish processing all commands
    println!("[Main] Waiting for main processing thread to finish...");
    let processing_wait_start = Instant::now();
    main_handle.join().expect("Main thread panicked");
    let processing_wait_time = processing_wait_start.elapsed();
    println!("[Main] Main processing thread finished");
    println!(
        "[Timing] Main thread completion took {:?}",
        processing_wait_time
    );

    let total_time = start_time.elapsed();
    println!("\n=== All processing complete! ===");
    println!("[Timing] Total execution time: {:?}", total_time);
    println!("\n--- Timing Breakdown ---");
    println!("  Pool creation:       {:?}", pool_creation_time);
    println!("  File reading:        {:?}", file_read_time);
    println!("  IO thread shutdown:  {:?}", shutdown_time);
    println!("  Processing wait:     {:?}", processing_wait_time);
    println!("  Total time:          {:?}", total_time);
}
