//! PostScript Interpreter Library
//!
//! This library implements a PostScript interpreter with support for dynamic and lexical scoping.
//!
//! # Architecture
//!
//! The interpreter is organized into four main modules that work together:
//!
//! - **types**: Core data structures (PostScriptValue, Context) that represent the interpreter state
//! - **parser**: Tokenizes and parses PostScript source code into PostScriptValue objects
//! - **interpreter**: Executes PostScriptValue objects using a stack-based execution model
//! - **commands**: Built-in PostScript command implementations (add, sub, if, for, etc.)
//!
//! # Data Flow
//!
//! 1. **Input** → **parser::Tokenizer** → Converts text into tokens
//! 2. **Tokens** → **parser::parse** → Converts tokens into PostScriptValue objects
//! 3. **Values** → **interpreter::Interpreter** → Executes values using Context
//! 4. **Context** → **commands** → Built-in functions manipulate Context state
//!
//! # Example
//!
//! ```rust
//! use postscript_interpreter::types::Context;
//! use postscript_interpreter::interpreter::Interpreter;
//! use postscript_interpreter::parser::{Tokenizer, parse};
//! use postscript_interpreter::commands::register_builtins;
//!
//! let mut context = Context::new(false); // false = dynamic scoping
//! register_builtins(&mut context);
//! let mut interpreter = Interpreter::new(context);
//!
//! let mut tokenizer = Tokenizer::new("3 4 add =");
//! let tokens = tokenizer.tokenize().unwrap();
//! let values = parse(tokens).unwrap();
//! interpreter.execute(values).unwrap();
//! ```

pub mod types;
pub mod parser;
pub mod interpreter;
pub mod commands;

