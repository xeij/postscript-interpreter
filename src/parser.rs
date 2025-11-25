use crate::types::PostScriptValue;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Int(i64),
    Real(f64),
    String(String),
    Name(String),
    LiteralName(String),
    LBracket,
    RBracket,
    LBrace,
    RBrace,
}

pub struct Tokenizer {
    input: Vec<char>,
    position: usize,
}

impl Tokenizer {
    pub fn new(input: &str) -> Self {
        Tokenizer {
            input: input.chars().collect(),
            position: 0,
        }
    }

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
                    if c.is_digit(10) || c == '-' || c == '+' || c == '.' {
                         // Potential number, but could be a name if malformed or just a sign
                         // PostScript is flexible. Let's try to parse as number, else name.
                         // Actually '-' is a name (sub), so we need to be careful.
                         // -1 is number. - is name.
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

    fn skip_whitespace(&mut self) {
        while self.position < self.input.len() && self.input[self.position].is_whitespace() {
            self.position += 1;
        }
    }

    fn skip_comment(&mut self) {
        while self.position < self.input.len() && self.input[self.position] != '\n' {
            self.position += 1;
        }
    }

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

    fn try_read_number(&mut self) -> Option<Token> {
        let start = self.position;
        // Check for sign
        if self.position < self.input.len() && (self.input[self.position] == '+' || self.input[self.position] == '-') {
            self.position += 1;
        }
        
        let mut has_digit = false;
        let mut has_dot = false;
        
        while self.position < self.input.len() {
            let c = self.input[self.position];
            if c.is_digit(10) {
                has_digit = true;
                self.position += 1;
            } else if c == '.' {
                if has_dot { break; } // Second dot
                has_dot = true;
                self.position += 1;
            } else {
                break;
            }
        }

        // If we consumed just a sign or nothing, it's not a number (unless it's just a sign, which is a name)
        // Actually, if we just consumed "-", it's a name, not a number.
        // So we need at least one digit or a dot (if .5)
        // But "." is not a number usually? PS supports .5
        
        if !has_digit && !has_dot {
            self.position = start;
            return None;
        }
        
        // If it was just "-" or "+" or "." with no digits?
        // "." is usually not a number in PS? "0." is. ".5" is.
        // "-" is a name.
        
        let s: String = self.input[start..self.position].iter().collect();
        
        // Check if the next char is a delimiter. If it's a regular char, then this might be part of a name like "123a"
        if self.position < self.input.len() {
            let c = self.input[self.position];
            if !c.is_whitespace() && !"()[]{}%/".contains(c) {
                 // It continues as a name
                 self.position = start;
                 return None;
            }
        }

        if has_dot {
            if let Ok(f) = s.parse::<f64>() {
                return Some(Token::Real(f));
            }
        } else {
            if let Ok(i) = s.parse::<i64>() {
                return Some(Token::Int(i));
            }
        }
        
        // Fallback
        self.position = start;
        None
    }
}

pub fn parse(tokens: Vec<Token>) -> Result<Vec<PostScriptValue>, String> {
    let mut iter = tokens.into_iter();
    parse_sequence(&mut iter, None)
}

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
                // Arrays in PS are usually constructed with [ ... ] which puts a mark, then values, then ] operator creates array.
                // However, for the parser, we might want to represent this structure if we are parsing a procedure.
                // But standard PS execution: [ is a name (mark). ] is a name (array creation).
                // Wait, `[` is an operator. `]` is an operator.
                // So we should parse them as Names if they are executable.
                // But wait, `{ ... }` IS a syntactic construct for executable arrays (procedures).
                // `[ ... ]` is NOT a syntactic construct for a single value in the same way, it's a sequence of operations.
                // EXCEPT: The user prompt says "copy has array, sequence, dictionary, string forms".
                // And "any[0] ... any[n-1] n copy".
                // The `[` and `]` are just operators.
                // HOWEVER, `{` and `}` create a procedure (executable array) IMMEDIATELY.
                
                // So:
                // `[` -> Name("[")
                // `]` -> Name("]")
                // `{` -> Start parsing procedure
                sequence.push(PostScriptValue::Name("[".to_string()));
            }
            Token::RBracket => {
                sequence.push(PostScriptValue::Name("]".to_string()));
            }
            Token::LBrace => {
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
