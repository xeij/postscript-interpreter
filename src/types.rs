//! Core Data Types for PostScript Interpreter
//!
//! This module defines the fundamental data structures used throughout the interpreter:
//! - `PostScriptValue`: Represents all possible PostScript values and execution states
//! - `Context`: Holds the complete interpreter state (stacks, dictionaries, scoping mode)

use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;
use std::cell::RefCell;

/// Represents all possible values and execution states in the PostScript interpreter.
///
/// This enum is the core data type that flows through the entire system:
/// - The parser converts tokens into PostScriptValue objects
/// - The interpreter executes PostScriptValue objects
/// - The operand stack stores PostScriptValue objects
/// - The execution stack contains PostScriptValue objects to be executed
#[derive(Debug, Clone, PartialEq)]
pub enum PostScriptValue {
    /// Integer number (e.g., 42, -17)
    Int(i64),
    
    /// Real (floating-point) number (e.g., 3.14, -2.5)
    Real(f64),
    
    /// Boolean value (true or false)
    Bool(bool),
    
    /// String literal (e.g., (hello world))
    /// Wrapped in Rc<RefCell<>> to support mutation (required for putinterval)
    String(Rc<RefCell<String>>),
    
    /// Executable name - a name that will be looked up and executed (e.g., add, sub, myfunction)
    Name(String),
    
    /// Literal name - a name used as data, not executed (e.g., /x, /myvar)
    /// Used as keys in dictionaries and for defining variables
    LiteralName(String),
    
    /// Array of values (e.g., [1 2 3])
    Array(Vec<PostScriptValue>),
    
    /// Dictionary - a hash map wrapped in Rc<RefCell<>> for shared mutable access
    /// Multiple references can point to the same dictionary (e.g., on dict stack)
    Dict(Rc<RefCell<HashMap<String, PostScriptValue>>>),
    
    /// Mark value used for array construction (the [ operator pushes this)
    Mark,
    
    /// Native Rust function that implements a built-in PostScript command
    /// Takes a mutable Context reference and returns Result
    NativeFn(fn(&mut Context) -> Result<(), String>),
    
    /// Executable array/procedure (e.g., { 1 2 add })
    /// In dynamic scoping, this is executed in the current environment
    Block(Vec<PostScriptValue>),
    
    // === Control Flow States ===
    // These variants represent active loop states on the execution stack
    
    /// Active for-loop state
    /// Stores current iteration value, step size, limit, and procedure to execute
    ForLoop { current: f64, step: f64, limit: f64, proc: Box<PostScriptValue> },
    
    /// Active repeat-loop state
    /// Stores remaining iteration count and procedure to execute
    RepeatLoop { count: i64, proc: Box<PostScriptValue> },
    
    // === Lexical Scoping Support ===
    
    /// Closure - a procedure with captured environment for lexical scoping
    /// Stores the procedure body and a snapshot of the dictionary stack at creation time
    Closure { body: Vec<PostScriptValue>, env: Vec<Rc<RefCell<HashMap<String, PostScriptValue>>>> },
    
    /// Marker to restore the dictionary stack after closure execution
    /// Used to restore the environment when a closure finishes executing
    RestoreEnv(Vec<Rc<RefCell<HashMap<String, PostScriptValue>>>>),
}

impl fmt::Display for PostScriptValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PostScriptValue::Int(i) => write!(f, "{}", i),
            PostScriptValue::Real(r) => write!(f, "{}", r),
            PostScriptValue::Bool(b) => write!(f, "{}", b),
            PostScriptValue::String(s) => write!(f, "({})", s.borrow()),
            PostScriptValue::Name(n) => write!(f, "{}", n),
            PostScriptValue::LiteralName(n) => write!(f, "/{}", n),
            PostScriptValue::Array(arr) => {
                write!(f, "[")?;
                for (i, v) in arr.iter().enumerate() {
                    if i > 0 { write!(f, " ")?; }
                    write!(f, "{}", v)?;
                }
                write!(f, "]")
            }
            PostScriptValue::Dict(_) => write!(f, "--nostringval--"),
            PostScriptValue::Mark => write!(f, "--mark--"),
            PostScriptValue::NativeFn(_) => write!(f, "--native-function--"),
            PostScriptValue::Block(arr) => {
                write!(f, "{{")?;
                for (i, v) in arr.iter().enumerate() {
                    if i > 0 { write!(f, " ")?; }
                    write!(f, "{}", v)?;
                }
                write!(f, "}}")
            }
            PostScriptValue::ForLoop { .. } => write!(f, "--for-loop--"),
            PostScriptValue::RepeatLoop { .. } => write!(f, "--repeat-loop--"),
            PostScriptValue::Closure { .. } => write!(f, "--closure--"),
            PostScriptValue::RestoreEnv(_) => write!(f, "--restore-env--"),
        }
    }
}

/// The complete interpreter state.
///
/// This structure holds all the runtime state needed to execute PostScript code:
/// - Operand stack: Where values are pushed/popped during computation
/// - Dictionary stack: Hierarchical namespace for variable lookup
/// - Execution stack: Queue of values waiting to be executed
/// - Scoping mode: Determines how closures capture their environment
///
/// # Communication with Other Modules
///
/// - **parser**: Creates PostScriptValue objects that get pushed to execution_stack
/// - **interpreter**: Pops from execution_stack, manipulates operand_stack and dict_stack
/// - **commands**: Built-in functions receive &mut Context to manipulate all stacks
pub struct Context {
    /// Operand stack - holds values during computation
    /// Commands pop arguments from here and push results back
    pub operand_stack: Vec<PostScriptValue>,
    
    /// Dictionary stack - hierarchical namespace for variable lookup
    /// Each dictionary is wrapped in Rc<RefCell<>> for shared mutable access
    /// Lookup searches from top to bottom (most recent to oldest)
    /// The bottom dictionary is the system dictionary with built-in commands
    pub dict_stack: Vec<Rc<RefCell<HashMap<String, PostScriptValue>>>>,
    
    /// Execution stack - holds values waiting to be executed
    /// The interpreter pops from this stack and executes each value
    /// Procedures and loops push their contents here for execution
    pub execution_stack: Vec<PostScriptValue>,
    
    /// Scoping mode flag
    /// - false: Dynamic scoping (variables resolved in calling context)
    /// - true: Lexical scoping (variables resolved in defining context)
    pub lexical_scoping: bool,
}

impl Context {
    /// Creates a new Context with the specified scoping mode.
    ///
    /// Initializes:
    /// - Empty operand stack
    /// - Dictionary stack with one system dictionary (for built-in commands)
    /// - Empty execution stack
    pub fn new(lexical_scoping: bool) -> Self {
        let system_dict = Rc::new(RefCell::new(HashMap::new()));
        Context {
            operand_stack: Vec::new(),
            dict_stack: vec![system_dict],
            execution_stack: Vec::new(),
            lexical_scoping,
        }
    }

    /// Pushes a value onto the operand stack.
    pub fn push(&mut self, val: PostScriptValue) {
        self.operand_stack.push(val);
    }

    /// Pops a value from the operand stack.
    /// Returns None if the stack is empty.
    pub fn pop(&mut self) -> Option<PostScriptValue> {
        self.operand_stack.pop()
    }

    /// Peeks at the top value on the operand stack without removing it.
    /// Returns None if the stack is empty.
    pub fn peek(&self) -> Option<&PostScriptValue> {
        self.operand_stack.last()
    }
    
    /// Defines a key-value pair in the current (topmost) dictionary.
    ///
    /// Used by the `def` command to create or update variables.
    /// The definition goes into the dictionary at the top of the dict_stack.
    pub fn define(&mut self, key: String, value: PostScriptValue) {
        if let Some(dict) = self.dict_stack.last() {
            dict.borrow_mut().insert(key, value);
        }
    }

    /// Looks up a name in the dictionary stack.
    ///
    /// Searches from top to bottom (most recent to oldest dictionary).
    /// Returns the first matching value found, or None if not found.
    ///
    /// This implements PostScript's hierarchical namespace:
    /// - Local definitions (in top dictionaries) shadow global ones
    /// - Built-in commands (in system dictionary at bottom) are always available
    pub fn lookup(&self, key: &str) -> Option<PostScriptValue> {
        for dict in self.dict_stack.iter().rev() {
            if let Some(val) = dict.borrow().get(key) {
                return Some(val.clone());
            }
        }
        None
    }
}

