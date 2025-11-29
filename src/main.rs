//! PostScript Interpreter CLI
//!
//! This is the command-line interface for the PostScript interpreter.
//! It supports both interactive REPL mode and file execution mode.

use std::env;
use std::fs;
use std::io::{self, Write};
use postscript_interpreter::types::Context;
use postscript_interpreter::interpreter::Interpreter;
use postscript_interpreter::parser::{Tokenizer, parse};
use postscript_interpreter::commands::register_builtins;

/// Main entry point for the PostScript interpreter CLI.
///
/// Parses command-line arguments to determine:
/// - Scoping mode (--lexical flag enables lexical scoping, default is dynamic)
/// - Input mode (file path for script execution, or REPL if no file provided)
///
/// # Example Usage
///
/// ```bash
/// # Interactive REPL with dynamic scoping
/// cargo run
///
/// # Execute script with dynamic scoping
/// cargo run -- script.ps
///
/// # Execute script with lexical scoping
/// cargo run -- --lexical script.ps
/// ```
fn main() {
    let args: Vec<String> = env::args().collect();
    let mut lexical_scoping = false;
    let mut input_file = None;

    // Parse command-line arguments
    for arg in args.iter().skip(1) {
        if arg == "--lexical" {
            lexical_scoping = true;
        } else {
            input_file = Some(arg);
        }
    }

    // Initialize the interpreter context with the chosen scoping mode
    let mut context = Context::new(lexical_scoping);
    
    // Register all built-in PostScript commands (add, sub, if, for, etc.)
    register_builtins(&mut context);
    
    // Create the interpreter with the configured context
    let mut interpreter = Interpreter::new(context);

    // Choose execution mode based on whether a file was provided
    if let Some(filename) = input_file {
        // File execution mode
        let content = fs::read_to_string(filename).expect("Could not read file");
        run(&mut interpreter, &content);
    } else {
        // Interactive REPL mode
        repl(&mut interpreter);
    }
}

/// Executes PostScript code through the complete pipeline:
/// 1. Tokenization: Converts source text into tokens
/// 2. Parsing: Converts tokens into PostScriptValue objects
/// 3. Execution: Runs the values through the interpreter
///
/// Errors at any stage are reported to stderr with appropriate context.
fn run(interpreter: &mut Interpreter, input: &str) {
    let mut tokenizer = Tokenizer::new(input);
    match tokenizer.tokenize() {
        Ok(tokens) => {
            match parse(tokens) {
                Ok(values) => {
                    if let Err(e) = interpreter.execute(values) {
                        eprintln!("Runtime Error: {}", e);
                    }
                }
                Err(e) => eprintln!("Parse Error: {}", e),
            }
        }
        Err(e) => eprintln!("Tokenization Error: {}", e),
    }
}

/// Interactive Read-Eval-Print Loop (REPL).
///
/// Continuously reads lines from stdin, executes them, and displays results.
/// The interpreter state persists across lines, so variables and definitions
/// remain available throughout the session.
///
/// Type 'quit' or press Ctrl+D to exit.
fn repl(interpreter: &mut Interpreter) {
    println!("PostScript Interpreter (Rust)");
    println!("Type 'quit' to exit.");
    
    loop {
        print!("PS> ");
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(n) => {
                if n == 0 { break; } // EOF (Ctrl+D)
                run(interpreter, &input);
            }
            Err(error) => {
                eprintln!("error: {}", error);
                break;
            }
        }
    }
}

