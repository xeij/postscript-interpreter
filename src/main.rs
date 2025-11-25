use std::env;
use std::fs;
use std::io::{self, Write};
use postscript_interpreter::types::Context;
use postscript_interpreter::interpreter::Interpreter;
use postscript_interpreter::parser::{Tokenizer, parse};
use postscript_interpreter::commands::register_builtins;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut lexical_scoping = false;
    let mut input_file = None;

    for arg in args.iter().skip(1) {
        if arg == "--lexical" {
            lexical_scoping = true;
        } else {
            input_file = Some(arg);
        }
    }

    let mut context = Context::new(lexical_scoping);
    register_builtins(&mut context);
    let mut interpreter = Interpreter::new(context);

    if let Some(filename) = input_file {
        let content = fs::read_to_string(filename).expect("Could not read file");
        run(&mut interpreter, &content);
    } else {
        repl(&mut interpreter);
    }
}

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

fn repl(interpreter: &mut Interpreter) {
    println!("PostScript Interpreter (Rust)");
    println!("Type 'quit' to exit.");
    
    loop {
        print!("PS> ");
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(n) => {
                if n == 0 { break; }
                run(interpreter, &input);
            }
            Err(error) => {
                eprintln!("error: {}", error);
                break;
            }
        }
    }
}
