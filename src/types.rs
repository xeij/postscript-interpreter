use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Debug, Clone, PartialEq)]
pub enum PostScriptValue {
    Int(i64),
    Real(f64),
    Bool(bool),
    String(String),
    Name(String), // Executable name (e.g., add)
    LiteralName(String), // Literal name (e.g., /x)
    Array(Vec<PostScriptValue>),
    Dict(Rc<RefCell<HashMap<String, PostScriptValue>>>),
    Mark,
    NativeFn(fn(&mut Context) -> Result<(), String>),
    Block(Vec<PostScriptValue>), // For executable arrays/procedures
    // Control flow states
    ForLoop { current: f64, step: f64, limit: f64, proc: Box<PostScriptValue> },
    RepeatLoop { count: i64, proc: Box<PostScriptValue> },
}

impl fmt::Display for PostScriptValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PostScriptValue::Int(i) => write!(f, "{}", i),
            PostScriptValue::Real(r) => write!(f, "{}", r),
            PostScriptValue::Bool(b) => write!(f, "{}", b),
            PostScriptValue::String(s) => write!(f, "({})", s),
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
        }
    }
}

pub struct Context {
    pub operand_stack: Vec<PostScriptValue>,
    pub dict_stack: Vec<Rc<RefCell<HashMap<String, PostScriptValue>>>>,
    pub execution_stack: Vec<PostScriptValue>, // For loops and procedures
    pub lexical_scoping: bool,
}

impl Context {
    pub fn new(lexical_scoping: bool) -> Self {
        let system_dict = Rc::new(RefCell::new(HashMap::new()));
        Context {
            operand_stack: Vec::new(),
            dict_stack: vec![system_dict],
            execution_stack: Vec::new(),
            lexical_scoping,
        }
    }

    pub fn push(&mut self, val: PostScriptValue) {
        self.operand_stack.push(val);
    }

    pub fn pop(&mut self) -> Option<PostScriptValue> {
        self.operand_stack.pop()
    }

    pub fn peek(&self) -> Option<&PostScriptValue> {
        self.operand_stack.last()
    }
    
    pub fn define(&mut self, key: String, value: PostScriptValue) {
        if let Some(dict) = self.dict_stack.last() {
            dict.borrow_mut().insert(key, value);
        }
    }

    pub fn lookup(&self, key: &str) -> Option<PostScriptValue> {
        for dict in self.dict_stack.iter().rev() {
            if let Some(val) = dict.borrow().get(key) {
                return Some(val.clone());
            }
        }
        None
    }
}
