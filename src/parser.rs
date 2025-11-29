//! PostScript Parser Module
//!
//! This module handles the conversion of PostScript source code into executable values.
//! It operates in two stages:
//! 1. Tokenization: Converts raw text into tokens
//! 2. Parsing: Converts tokens into PostScriptValue objects

use crate::types::PostScriptValue;


/// Represents a lexical token in PostScript source code.
///
/// Tokens are the atomic units produced by the tokenizer before parsing.
#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    /// Integer literal (e.g., 42, -17)
    Int(i64),
    /// Real number literal (e.g., 3.14, -2.5)
    Real(f64),
    /// String literal (e.g., (hello world))
    String(String),
    /// Executable name (e.g., add, sub, myfunction)
    Name(String),
    /// Literal name starting with / (e.g., /x, /myvar)
    LiteralName(String),
    /// Left bracket [ (used as an operator in PostScript)
    LBracket,
    /// Right bracket ] (used as an operator in PostScript)
    RBracket,
    /// Left brace { (starts a procedure/block)
    LBrace,
    /// Right brace } (ends a procedure/block)
    RBrace,
}

/// Tokenizer converts PostScript source text into a sequence of tokens.
///
/// The tokenizer handles:
/// - Numbers (integers and reals)
/// - Strings with escape sequences
/// - Names (executable and literal)
/// - Brackets and braces
/// - Comments (% to end of line)
/// - Whitespace
pub struct Tokenizer {
    input: Vec<char>,
    position: usize,
}

impl Tokenizer {
    /// Creates a new tokenizer for the given input string.
    pub fn new(input: &str) -> Self {
        Tokenizer {
            input: input.chars().collect(),
            position: 0,
        }
    }

    /// Tokenizes the entire input string into a vector of tokens.
    ///
    /// Returns an error if the input contains invalid syntax (e.g., unterminated string).
    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();
        while self.position < self.input.len() {
            self.skip_whitespace();
            if self.position >= self.input.len() {
                break;
            }

            let c = self.input[self.position];
            match c {
                '%' => self.skip_comment(),
                '(' => tokens.push(self.read_string()?),
                '[' => {
                    tokens.push(Token::LBracket);
                    self.position += 1;
                }
                ']' => {
                    tokens.push(Token::RBracket);
                    self.position += 1;
                }
                '{' => {
                    tokens.push(Token::LBrace);
                    self.position += 1;
                }
                '}' => {
                    tokens.push(Token::RBrace);
                    self.position += 1;
                }
                '/' => tokens.push(self.read_literal_name()?),
                _ => {
                    // Try to parse as number first, otherwise treat as name
                    if c.is_digit(10) || c == '-' || c == '+' || c == '.' {
                         if let Some(tok) = self.try_read_number() {
                             tokens.push(tok);
                         } else {
                             tokens.push(self.read_name()?);
                         }
                    } else {
                        tokens.push(self.read_name()?);
                    }
                }
            }
        }
        Ok(tokens)
    }

    /// Skips whitespace characters (space, tab, newline, etc.).
    fn skip_whitespace(&mut self) {
        while self.position < self.input.len() && self.input[self.position].is_whitespace() {
            self.position += 1;
        }
    }

    /// Skips a comment (from % to end of line).
    fn skip_comment(&mut self) {
        while self.position < self.input.len() && self.input[self.position] != '\n' {
            self.position += 1;
        }
    }

    /// Reads a string literal enclosed in parentheses.
    ///
    /// Handles:
    /// - Nested parentheses (strings can contain balanced parens)
    /// - Escape sequences (\n, \r, \t, \\, \(, \), etc.)
    fn read_string(&mut self) -> Result<Token, String> {
        self.position += 1; // Skip '('
        let mut s = String::new();
        let mut depth = 1;
        
        while self.position < self.input.len() {
            let c = self.input[self.position];
            match c {
                '(' => {
                    depth += 1;
                    s.push(c);
                }
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        self.position += 1;
                        return Ok(Token::String(s));
                    }
                    s.push(c);
                }
                '\\' => {
                    self.position += 1;
                    if self.position >= self.input.len() {
                        return Err("Unexpected end of input in string".to_string());
                    }
                    let escaped = self.input[self.position];
                    match escaped {
                        'n' => s.push('\n'),
                        'r' => s.push('\r'),
                        't' => s.push('\t'),
                        'b' => s.push('\x08'),
                        'f' => s.push('\x0c'),
                        '\\' => s.push('\\'),
                        '(' => s.push('('),
                        ')' => s.push(')'),
                        _ => s.push(escaped), // Fallback
                    }
                }
                _ => s.push(c),
            }
            self.position += 1;
        }
        Err("Unterminated string".to_string())
    }

    /// Reads a literal name (starts with /).
    ///
    /// Literal names are used as keys in dictionaries and for variable definitions.
    /// Example: /x, /myvar, /add
    fn read_literal_name(&mut self) -> Result<Token, String> {
        self.position += 1; // Skip '/'
        let start = self.position;
        while self.position < self.input.len() {
            let c = self.input[self.position];
            if c.is_whitespace() || "()[]{}%/".contains(c) {
                break;
            }
            self.position += 1;
        }
        let name: String = self.input[start..self.position].iter().collect();
        Ok(Token::LiteralName(name))
    }

    /// Reads an executable name (no leading /).
    ///
    /// Executable names are looked up and executed.
    /// Example: add, sub, myfunction
    fn read_name(&mut self) -> Result<Token, String> {
        let start = self.position;
        while self.position < self.input.len() {
            let c = self.input[self.position];
            if c.is_whitespace() || "()[]{}%/".contains(c) {
                break;
            }
            self.position += 1;
        }
        let name: String = self.input[start..self.position].iter().collect();
        Ok(Token::Name(name))
    }

    /// Attempts to read a number (integer or real).
    ///
    /// Returns None if the text doesn't form a valid number.
    /// This allows fallback to name parsing for things like "-" or "+".
    ///
    /// Handles:
    /// - Optional sign (+/-)
    /// - Integer literals (e.g., 42, -17)
    /// - Real literals (e.g., 3.14, -2.5, .5)
    /// - Distinguishes numbers from names (e.g., "123" vs "123abc")
    fn try_read_number(&mut self) -> Option<Token> {
        let start = self.position;
        
        // Check for optional sign
        if self.position < self.input.len() && (self.input[self.position] == '+' || self.input[self.position] == '-') {
            self.position += 1;
        }
        
        let mut has_digit = false;
        let mut has_dot = false;
        
        // Read digits and optional decimal point
        while self.position < self.input.len() {
            let c = self.input[self.position];
            if c.is_digit(10) {
                has_digit = true;
                self.position += 1;
            } else if c == '.' {
                if has_dot { break; } // Second dot means end of number
                has_dot = true;
                self.position += 1;
            } else {
                break;
            }
        }

        // Need at least one digit to be a valid number
        if !has_digit && !has_dot {
            self.position = start;
            return None;
        }
        
        let s: String = self.input[start..self.position].iter().collect();
        
        // Verify the next character is a delimiter (not part of a name)
        if self.position < self.input.len() {
            let c = self.input[self.position];
            if !c.is_whitespace() && !"()[]{}%/".contains(c) {
                 // Continues as a name (e.g., "123abc")
                 self.position = start;
                 return None;
            }
        }

        // Parse as real or integer
        if has_dot {
            if let Ok(f) = s.parse::<f64>() {
                return Some(Token::Real(f));
            }
        } else {
            if let Ok(i) = s.parse::<i64>() {
                return Some(Token::Int(i));
            }
        }
        
        // Parsing failed, treat as name
        self.position = start;
        None
    }
}

/// Parses a sequence of tokens into PostScriptValue objects.
///
/// This is the main entry point for parsing. It converts the flat token stream
/// into a structured representation with nested blocks for procedures.
///
/// # Communication with Interpreter
///
/// The resulting Vec<PostScriptValue> is passed to the interpreter's execute() method,
/// which pushes these values onto the execution stack for processing.
pub fn parse(tokens: Vec<Token>) -> Result<Vec<PostScriptValue>, String> {
    let mut iter = tokens.into_iter();
    parse_sequence(&mut iter, None)
}

/// Recursively parses a sequence of tokens until a terminator is found.
///
/// This function handles:
/// - Converting tokens to PostScriptValue objects
/// - Recursively parsing blocks ({ ... }) into Block values
/// - Treating [ and ] as executable names (operators)
///
/// The terminator parameter is used when parsing blocks to know when to stop.
fn parse_sequence(iter: &mut std::vec::IntoIter<Token>, terminator: Option<Token>) -> Result<Vec<PostScriptValue>, String> {
    let mut sequence = Vec::new();
    while let Some(token) = iter.next() {
        if let Some(ref term) = terminator {
            if token == *term {
                return Ok(sequence);
            }
        }
        
        match token {
            Token::Int(i) => sequence.push(PostScriptValue::Int(i)),
            Token::Real(f) => sequence.push(PostScriptValue::Real(f)),
            Token::String(s) => sequence.push(PostScriptValue::String(s)),
            Token::Name(n) => sequence.push(PostScriptValue::Name(n)),
            Token::LiteralName(n) => sequence.push(PostScriptValue::LiteralName(n)),
            Token::LBracket => {
                // [ is treated as an executable name (operator)
                // In PostScript, [ pushes a mark on the stack
                sequence.push(PostScriptValue::Name("[".to_string()));
            }
            Token::RBracket => {
                // ] is treated as an executable name (operator)
                // In PostScript, ] creates an array from items above the mark
                sequence.push(PostScriptValue::Name("]".to_string()));
            }
            Token::LBrace => {
                // { starts a procedure/block - parse until matching }
                // The contents become a Block value (executable array)
                let block = parse_sequence(iter, Some(Token::RBrace))?;
                sequence.push(PostScriptValue::Block(block));
            }
            Token::RBrace => {
                return Err("Unexpected }".to_string());
            }
        }
    }
    
    if terminator.is_some() {
        return Err("Unexpected end of input, expected terminator".to_string());
    }
    
    Ok(sequence)
}
