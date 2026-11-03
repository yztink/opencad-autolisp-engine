use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process;

use lisp_core::{create_default_env, eval_str, Value};
use lisp_repl::{ReplInputStatus, ReplSession};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        // Run specified script file
        let file_path = &args[1];
        if !Path::new(file_path).exists() {
            eprintln!("Error: File not found: {}", file_path);
            process::exit(1);
        }

        match fs::read_to_string(file_path) {
            Ok(code) => {
                let mut env = create_default_env();
                match eval_str(&code, &mut env) {
                    Ok(val) => {
                        if val != Value::Nil {
                            println!("{}", val);
                        }
                    }
                    Err(err) => {
                        eprintln!("AutoLISP Error: {}", err);
                        process::exit(1);
                    }
                }
            }
            Err(e) => {
                eprintln!("Error reading file '{}': {}", file_path, e);
                process::exit(1);
            }
        }
    } else {
        // Interactive multiline REPL
        run_repl();
    }
}

fn run_repl() {
    println!("OpenCAD AutoLISP Engine (Rust) v0.1.0");
    println!("Type AutoLISP expressions to evaluate. Multiline parentheses are balanced automatically.\nType 'quit' or 'exit' to exit.\n");

    let mut session = ReplSession::new();
    let stdin = io::stdin();

    loop {
        if session.buffer().is_empty() {
            print!("_$ ");
        } else {
            print!("   ");
        }
        let _ = io::stdout().flush();

        let mut line = String::new();
        if stdin.read_line(&mut line).is_err() || line.is_empty() {
            println!("\nBye.");
            break;
        }

        let trimmed = line.trim();
        if session.buffer().is_empty() && (trimmed == "exit" || trimmed == "quit") {
            break;
        }

        let (status, output) = session.feed_line(&line);
        match status {
            ReplInputStatus::Complete => {
                if let Some(out) = output {
                    if out.success {
                        if let Some(res) = out.result {
                            println!("{}", res);
                        }
                    } else if let Some(err) = out.error {
                        eprintln!("; error: {}", err);
                    }
                }
            }
            ReplInputStatus::Incomplete => {
                // Continuation line requested
            }
            ReplInputStatus::Empty => {}
        }
    }
}
