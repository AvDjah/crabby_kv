mod handler;
mod parser;

use std::{
    fs::File,
    io::{BufRead, BufReader},
};

fn main() {
    println!("Starting command processor...");

    let mut handler = handler::CommandHandler::new();

    let f = File::open("input.txt");
    match f {
        Ok(file) => {
            let reader = BufReader::new(file);
            let mut line_num = 0;

            for line_result in reader.lines() {
                match line_result {
                    Ok(line) => {
                        line_num += 1;

                        // Skip empty lines silently
                        if line.trim().is_empty() {
                            continue;
                        }

                        // Try to parse the command
                        match line.parse::<parser::Command>() {
                            Ok(command) => {
                                // Process the command
                                match handler.process_command(command) {
                                    Ok(response) => println!("[{}] {}", line_num, response),
                                    Err(err) => eprintln!("[{}] Error: {}", line_num, err),
                                }
                            }
                            Err(parse_err) => {
                                eprintln!(
                                    "[{}] Parse Error: {} (line: '{}')",
                                    line_num, parse_err, line
                                );
                            }
                        }
                    }
                    Err(err) => {
                        eprintln!("Error reading line: {}", err);
                        break;
                    }
                }
            }

            println!("\nProcessed {} lines", line_num);
        }
        Err(err) => {
            panic!("Error opening input file: {}", err);
        }
    }
}
