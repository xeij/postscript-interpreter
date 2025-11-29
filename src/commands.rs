//! Built-in PostScript Command Implementations
//!
//! This module contains all the native PostScript command implementations.
//! Each command is a Rust function that takes `&mut Context` and returns `Result<(), String>`.
//!
//! # Command Categories
//!
//! - **Stack Manipulation**: exch, pop, copy, dup, clear, count
//! - **Arithmetic**: add, sub, mul, div, idiv, mod, abs, neg, ceiling, floor, round, sqrt
//! - **Dictionary**: dict, length, maxlength, begin, end, def
//! - **String**: get, getinterval, putinterval
//! - **Boolean/Bit**: eq, ne, ge, gt, le, lt, and, or, not
//! - **Flow Control**: if, ifelse, for, repeat, quit
//! - **I/O**: print, =, ==
//!
//! # How Commands Work
//!
//! Commands manipulate the Context state:
//! 1. Pop arguments from the operand stack
//! 2. Perform the operation
//! 3. Push results back to the operand stack
//! 4. Return Ok(()) on success or Err(message) on failure
//!
//! The interpreter calls these functions when it encounters a Name that maps to a NativeFn.

use crate::types::{Context, PostScriptValue};
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

/// Registers all built-in PostScript commands in the given context.
///
/// This function is called during interpreter initialization to populate the
/// system dictionary with all native commands. Each command is registered as
/// a NativeFn value that points to the corresponding Rust function.
///
/// # Example
///
/// ```ignore
/// let mut context = Context::new(false);
/// register_builtins(&mut context);
/// // Now context.lookup("add") returns Some(NativeFn(add))
/// ```
pub fn register_builtins(context: &mut Context) {
    // Stack Manipulation
    context.define("exch".to_string(), PostScriptValue::NativeFn(exch));
    context.define("pop".to_string(), PostScriptValue::NativeFn(pop));
    context.define("copy".to_string(), PostScriptValue::NativeFn(copy));
    context.define("dup".to_string(), PostScriptValue::NativeFn(dup));
    context.define("clear".to_string(), PostScriptValue::NativeFn(clear));
    context.define("count".to_string(), PostScriptValue::NativeFn(count));

    // Arithmetic
    context.define("add".to_string(), PostScriptValue::NativeFn(add));
    context.define("sub".to_string(), PostScriptValue::NativeFn(sub));
    context.define("mul".to_string(), PostScriptValue::NativeFn(mul));
    context.define("div".to_string(), PostScriptValue::NativeFn(div));
    context.define("idiv".to_string(), PostScriptValue::NativeFn(idiv));
    context.define("mod".to_string(), PostScriptValue::NativeFn(mod_op));
    context.define("abs".to_string(), PostScriptValue::NativeFn(abs));
    context.define("neg".to_string(), PostScriptValue::NativeFn(neg));
    context.define("ceiling".to_string(), PostScriptValue::NativeFn(ceiling));
    context.define("floor".to_string(), PostScriptValue::NativeFn(floor));
    context.define("round".to_string(), PostScriptValue::NativeFn(round));
    context.define("sqrt".to_string(), PostScriptValue::NativeFn(sqrt));

    // Dictionary
    context.define("dict".to_string(), PostScriptValue::NativeFn(dict));
    context.define("length".to_string(), PostScriptValue::NativeFn(length));
    context.define("maxlength".to_string(), PostScriptValue::NativeFn(maxlength));
    context.define("begin".to_string(), PostScriptValue::NativeFn(begin));
    context.define("end".to_string(), PostScriptValue::NativeFn(end));
    context.define("def".to_string(), PostScriptValue::NativeFn(def));

    // String
    context.define("get".to_string(), PostScriptValue::NativeFn(get));
    context.define("getinterval".to_string(), PostScriptValue::NativeFn(getinterval));
    context.define("putinterval".to_string(), PostScriptValue::NativeFn(putinterval));

    // Boolean/Bit
    context.define("eq".to_string(), PostScriptValue::NativeFn(eq));
    context.define("ne".to_string(), PostScriptValue::NativeFn(ne));
    context.define("ge".to_string(), PostScriptValue::NativeFn(ge));
    context.define("gt".to_string(), PostScriptValue::NativeFn(gt));
    context.define("le".to_string(), PostScriptValue::NativeFn(le));
    context.define("lt".to_string(), PostScriptValue::NativeFn(lt));
    context.define("and".to_string(), PostScriptValue::NativeFn(and));
    context.define("not".to_string(), PostScriptValue::NativeFn(not));
    context.define("or".to_string(), PostScriptValue::NativeFn(or));
    context.define("true".to_string(), PostScriptValue::Bool(true));
    context.define("false".to_string(), PostScriptValue::Bool(false));

    // Flow Control
    context.define("if".to_string(), PostScriptValue::NativeFn(if_op));
    context.define("ifelse".to_string(), PostScriptValue::NativeFn(ifelse));
    context.define("for".to_string(), PostScriptValue::NativeFn(for_op));
    context.define("repeat".to_string(), PostScriptValue::NativeFn(repeat));
    context.define("quit".to_string(), PostScriptValue::NativeFn(quit));

    // I/O
    context.define("print".to_string(), PostScriptValue::NativeFn(print));
    context.define("=".to_string(), PostScriptValue::NativeFn(eq_print));
    context.define("==".to_string(), PostScriptValue::NativeFn(eqeq_print));
}

// ============================================================================
// Stack Manipulation Commands
// ============================================================================

/// exch: Exchange the top two items on the stack
/// Stack: any1 any2 → any2 any1
fn exch(ctx: &mut Context) -> Result<(), String> {
    if ctx.operand_stack.len() < 2 {
        return Err("Stack underflow".to_string());
    }
    let len = ctx.operand_stack.len();
    ctx.operand_stack.swap(len - 1, len - 2);
    Ok(())
}

/// pop: Remove the top item from the stack
/// Stack: any → (empty)
fn pop(ctx: &mut Context) -> Result<(), String> {
    ctx.pop().ok_or("Stack underflow".to_string())?;
    Ok(())
}

/// copy: Copy the top n items on the stack
/// Stack: any[0] ... any[n-1] n → any[0] ... any[n-1] any[0] ... any[n-1]
/// 
/// Note: Object copy forms (dict/array/string copy) are not implemented.
/// Only stack copy (n items) is supported.
fn copy(ctx: &mut Context) -> Result<(), String> {
    let top = ctx.pop().ok_or("Stack underflow".to_string())?;
    match top {
        PostScriptValue::Int(n) => {
            // Stack copy: duplicate the top n items
            let n = n as usize;
            if ctx.operand_stack.len() < n {
                return Err("Stack underflow".to_string());
            }
            let len = ctx.operand_stack.len();
            for i in 0..n {
                let val = ctx.operand_stack[len - n + i].clone();
                ctx.push(val);
            }
        }
        _ => {
            // Object copy forms (dict/array/string) are not implemented
            match top {
                PostScriptValue::Dict(_) | PostScriptValue::String(_) | PostScriptValue::Array(_) => {
                    let _src = ctx.pop().ok_or("Stack underflow".to_string())?;
                    return Err("Object copy not fully implemented".to_string());
                }
                _ => return Err("Type check error: copy expected int".to_string()),
            }
        }
    }
    Ok(())
}

/// dup: Duplicate the top item on the stack
/// Stack: any → any any
fn dup(ctx: &mut Context) -> Result<(), String> {
    let val = ctx.peek().ok_or("Stack underflow".to_string())?.clone();
    ctx.push(val);
    Ok(())
}

/// clear: Remove all items from the operand stack
/// Stack: any[1] ... any[n] → (empty)
fn clear(ctx: &mut Context) -> Result<(), String> {
    ctx.operand_stack.clear();
    Ok(())
}

/// count: Push the number of items on the stack
/// Stack: any[1] ... any[n] → any[1] ... any[n] n
fn count(ctx: &mut Context) -> Result<(), String> {
    let n = ctx.operand_stack.len() as i64;
    ctx.push(PostScriptValue::Int(n));
    Ok(())
}

// ============================================================================
// Arithmetic Operations
// ============================================================================

/// add: Add two numbers
/// Stack: num1 num2 → num1+num2
/// Supports int+int, real+real, and mixed types (result is real if either operand is real)
fn add(ctx: &mut Context) -> Result<(), String> {
    let b = ctx.pop().ok_or("Stack underflow".to_string())?;
    let a = ctx.pop().ok_or("Stack underflow".to_string())?;
    match (a, b) {
        (PostScriptValue::Int(i1), PostScriptValue::Int(i2)) => ctx.push(PostScriptValue::Int(i1 + i2)),
        (PostScriptValue::Real(f1), PostScriptValue::Real(f2)) => ctx.push(PostScriptValue::Real(f1 + f2)),
        (PostScriptValue::Int(i1), PostScriptValue::Real(f2)) => ctx.push(PostScriptValue::Real(i1 as f64 + f2)),
        (PostScriptValue::Real(f1), PostScriptValue::Int(i2)) => ctx.push(PostScriptValue::Real(f1 + i2 as f64)),
        _ => return Err("Type check error".to_string()),
    }
    Ok(())
}

/// sub: Subtract two numbers
/// Stack: num1 num2 → num1-num2
fn sub(ctx: &mut Context) -> Result<(), String> {
    let b = ctx.pop().ok_or("Stack underflow".to_string())?;
    let a = ctx.pop().ok_or("Stack underflow".to_string())?;
    match (a, b) {
        (PostScriptValue::Int(i1), PostScriptValue::Int(i2)) => ctx.push(PostScriptValue::Int(i1 - i2)),
        (PostScriptValue::Real(f1), PostScriptValue::Real(f2)) => ctx.push(PostScriptValue::Real(f1 - f2)),
        (PostScriptValue::Int(i1), PostScriptValue::Real(f2)) => ctx.push(PostScriptValue::Real(i1 as f64 - f2)),
        (PostScriptValue::Real(f1), PostScriptValue::Int(i2)) => ctx.push(PostScriptValue::Real(f1 - i2 as f64)),
        _ => return Err("Type check error".to_string()),
    }
    Ok(())
}

/// mul: Multiply two numbers
/// Stack: num1 num2 → num1*num2
fn mul(ctx: &mut Context) -> Result<(), String> {
    let b = ctx.pop().ok_or("Stack underflow".to_string())?;
    let a = ctx.pop().ok_or("Stack underflow".to_string())?;
    match (a, b) {
        (PostScriptValue::Int(i1), PostScriptValue::Int(i2)) => ctx.push(PostScriptValue::Int(i1 * i2)),
        (PostScriptValue::Real(f1), PostScriptValue::Real(f2)) => ctx.push(PostScriptValue::Real(f1 * f2)),
        (PostScriptValue::Int(i1), PostScriptValue::Real(f2)) => ctx.push(PostScriptValue::Real(i1 as f64 * f2)),
        (PostScriptValue::Real(f1), PostScriptValue::Int(i2)) => ctx.push(PostScriptValue::Real(f1 * i2 as f64)),
        _ => return Err("Type check error".to_string()),
    }
    Ok(())
}

/// div: Divide two numbers (always returns real)
/// Stack: num1 num2 → num1/num2
fn div(ctx: &mut Context) -> Result<(), String> {
    let b = ctx.pop().ok_or("Stack underflow".to_string())?;
    let a = ctx.pop().ok_or("Stack underflow".to_string())?;
    match (a, b) {
        (PostScriptValue::Int(i1), PostScriptValue::Int(i2)) => ctx.push(PostScriptValue::Real(i1 as f64 / i2 as f64)),
        (PostScriptValue::Real(f1), PostScriptValue::Real(f2)) => ctx.push(PostScriptValue::Real(f1 / f2)),
        (PostScriptValue::Int(i1), PostScriptValue::Real(f2)) => ctx.push(PostScriptValue::Real(i1 as f64 / f2)),
        (PostScriptValue::Real(f1), PostScriptValue::Int(i2)) => ctx.push(PostScriptValue::Real(f1 / i2 as f64)),
        _ => return Err("Type check error".to_string()),
    }
    Ok(())
}

/// idiv: Integer division
/// Stack: int1 int2 → int1/int2 (truncated to integer)
fn idiv(ctx: &mut Context) -> Result<(), String> {
    let b = ctx.pop().ok_or("Stack underflow".to_string())?;
    let a = ctx.pop().ok_or("Stack underflow".to_string())?;
    match (a, b) {
        (PostScriptValue::Int(i1), PostScriptValue::Int(i2)) => ctx.push(PostScriptValue::Int(i1 / i2)),
        _ => return Err("Type check error".to_string()),
    }
    Ok(())
}

/// mod: Modulo operation
/// Stack: int1 int2 → int1 mod int2
fn mod_op(ctx: &mut Context) -> Result<(), String> {
    let b = ctx.pop().ok_or("Stack underflow".to_string())?;
    let a = ctx.pop().ok_or("Stack underflow".to_string())?;
    match (a, b) {
        (PostScriptValue::Int(i1), PostScriptValue::Int(i2)) => ctx.push(PostScriptValue::Int(i1 % i2)),
        _ => return Err("Type check error".to_string()),
    }
    Ok(())
}

/// abs: Absolute value
/// Stack: num → |num|
fn abs(ctx: &mut Context) -> Result<(), String> {
    let a = ctx.pop().ok_or("Stack underflow".to_string())?;
    match a {
        PostScriptValue::Int(i) => ctx.push(PostScriptValue::Int(i.abs())),
        PostScriptValue::Real(f) => ctx.push(PostScriptValue::Real(f.abs())),
        _ => return Err("Type check error".to_string()),
    }
    Ok(())
}

/// neg: Negation
/// Stack: num → -num
fn neg(ctx: &mut Context) -> Result<(), String> {
    let a = ctx.pop().ok_or("Stack underflow".to_string())?;
    match a {
        PostScriptValue::Int(i) => ctx.push(PostScriptValue::Int(-i)),
        PostScriptValue::Real(f) => ctx.push(PostScriptValue::Real(-f)),
        _ => return Err("Type check error".to_string()),
    }
    Ok(())
}

/// ceiling: Round up to nearest integer (returns real)
/// Stack: num → ⌈num⌉
fn ceiling(ctx: &mut Context) -> Result<(), String> {
    let a = ctx.pop().ok_or("Stack underflow".to_string())?;
    match a {
        PostScriptValue::Int(i) => ctx.push(PostScriptValue::Real(i as f64)), 
        PostScriptValue::Real(f) => ctx.push(PostScriptValue::Real(f.ceil())),
        _ => return Err("Type check error".to_string()),
    }
    Ok(())
}

/// floor: Round down to nearest integer (returns real)
/// Stack: num → ⌊num⌋
fn floor(ctx: &mut Context) -> Result<(), String> {
    let a = ctx.pop().ok_or("Stack underflow".to_string())?;
    match a {
        PostScriptValue::Int(i) => ctx.push(PostScriptValue::Real(i as f64)),
        PostScriptValue::Real(f) => ctx.push(PostScriptValue::Real(f.floor())),
        _ => return Err("Type check error".to_string()),
    }
    Ok(())
}

/// round: Round to nearest integer
/// Stack: num → round(num)
fn round(ctx: &mut Context) -> Result<(), String> {
    let a = ctx.pop().ok_or("Stack underflow".to_string())?;
    match a {
        PostScriptValue::Int(i) => ctx.push(PostScriptValue::Int(i)),
        PostScriptValue::Real(f) => ctx.push(PostScriptValue::Real(f.round())),
        _ => return Err("Type check error".to_string()),
    }
    Ok(())
}

/// sqrt: Square root
/// Stack: num → √num
fn sqrt(ctx: &mut Context) -> Result<(), String> {
    let a = ctx.pop().ok_or("Stack underflow".to_string())?;
    match a {
        PostScriptValue::Int(i) => ctx.push(PostScriptValue::Real((i as f64).sqrt())),
        PostScriptValue::Real(f) => ctx.push(PostScriptValue::Real(f.sqrt())),
        _ => return Err("Type check error".to_string()),
    }
    Ok(())
}

// ============================================================================
// Dictionary Operations
// ============================================================================

/// dict: Create a new dictionary
/// Stack: int → dict
/// Creates a dictionary with the specified initial capacity
fn dict(ctx: &mut Context) -> Result<(), String> {
    let a = ctx.pop().ok_or("Stack underflow".to_string())?;
    match a {
        PostScriptValue::Int(_) => {
            let d = Rc::new(RefCell::new(HashMap::new()));
            ctx.push(PostScriptValue::Dict(d));
        }
        _ => return Err("Type check error".to_string()),
    }
    Ok(())
}

/// length: Get the length of a composite object
/// Stack: dict|string|array → int
/// Returns the number of elements in the object
fn length(ctx: &mut Context) -> Result<(), String> {
    let a = ctx.pop().ok_or("Stack underflow".to_string())?;
    match a {
        PostScriptValue::Dict(d) => ctx.push(PostScriptValue::Int(d.borrow().len() as i64)),
        PostScriptValue::String(s) => ctx.push(PostScriptValue::Int(s.borrow().len() as i64)),
        PostScriptValue::Array(arr) => ctx.push(PostScriptValue::Int(arr.len() as i64)),
        PostScriptValue::Block(arr) => ctx.push(PostScriptValue::Int(arr.len() as i64)),
        PostScriptValue::Closure { body, .. } => ctx.push(PostScriptValue::Int(body.len() as i64)),
        _ => return Err("Type check error".to_string()),
    }
    Ok(())
}

/// maxlength: Get the capacity of a dictionary
/// Stack: dict → int
fn maxlength(ctx: &mut Context) -> Result<(), String> {
    let a = ctx.pop().ok_or("Stack underflow".to_string())?;
    match a {
        PostScriptValue::Dict(d) => ctx.push(PostScriptValue::Int(d.borrow().capacity() as i64)),
        _ => return Err("Type check error".to_string()),
    }
    Ok(())
}

/// begin: Push a dictionary onto the dictionary stack
/// Stack: dict → (empty)
/// Makes the dictionary the current context for variable lookups
fn begin(ctx: &mut Context) -> Result<(), String> {
    let a = ctx.pop().ok_or("Stack underflow".to_string())?;
    match a {
        PostScriptValue::Dict(d) => ctx.dict_stack.push(d),
        _ => return Err("Type check error".to_string()),
    }
    Ok(())
}

/// end: Pop the dictionary stack
/// Stack: (empty) → (empty)
/// Removes the current dictionary from the lookup context
fn end(ctx: &mut Context) -> Result<(), String> {
    if ctx.dict_stack.len() <= 1 { // Don't pop system dict
        return Err("Dict stack underflow".to_string());
    }
    ctx.dict_stack.pop();
    Ok(())
}

/// def: Define a key-value pair in the current dictionary
/// Stack: key value → (empty)
/// Associates the key with the value in the topmost dictionary
fn def(ctx: &mut Context) -> Result<(), String> {
    let value = ctx.pop().ok_or("Stack underflow".to_string())?;
    let key = ctx.pop().ok_or("Stack underflow".to_string())?;
    match key {
        PostScriptValue::Name(k) | PostScriptValue::LiteralName(k) => {
            ctx.define(k, value);
        }
        _ => return Err("Type check error: def expected name key".to_string()),
    }
    Ok(())
}

// ============================================================================
// String Operations
// ============================================================================

/// get: Get an element from a string or array
/// Stack: string|array index → int|any
/// For strings, returns the ASCII value of the character at the index
/// For arrays, returns the element at the index
fn get(ctx: &mut Context) -> Result<(), String> {
    let index = ctx.pop().ok_or("Stack underflow".to_string())?;
    let container = ctx.pop().ok_or("Stack underflow".to_string())?;
    match (container, index) {
        (PostScriptValue::String(s), PostScriptValue::Int(i)) => {
            let s_borrowed = s.borrow();
            if i < 0 || i as usize >= s_borrowed.len() {
                return Err("Range check error".to_string());
            }
            let c = s_borrowed.chars().nth(i as usize).unwrap();
            ctx.push(PostScriptValue::Int(c as i64));
        }
        (PostScriptValue::Array(arr), PostScriptValue::Int(i)) => {
             if i < 0 || i as usize >= arr.len() {
                return Err("Range check error".to_string());
            }
            ctx.push(arr[i as usize].clone());
        }
        _ => return Err("Type check error".to_string()),
    }
    Ok(())
}

/// getinterval: Extract a substring or subarray
/// Stack: string|array index count → substring|subarray
fn getinterval(ctx: &mut Context) -> Result<(), String> {
    let count = ctx.pop().ok_or("Stack underflow".to_string())?;
    let index = ctx.pop().ok_or("Stack underflow".to_string())?;
    let container = ctx.pop().ok_or("Stack underflow".to_string())?;
    
    match (container, index, count) {
        (PostScriptValue::String(s), PostScriptValue::Int(i), PostScriptValue::Int(c)) => {
            let i = i as usize;
            let c = c as usize;
            let s_borrowed = s.borrow();
            if i + c > s_borrowed.len() {
                return Err("Range check error".to_string());
            }
            let sub = s_borrowed[i..i+c].to_string();
            ctx.push(PostScriptValue::String(Rc::new(RefCell::new(sub))));
        }
        _ => return Err("Type check error".to_string()),
    }
    Ok(())
}

/// putinterval: Replace part of a string with another string
/// Stack: string1 index string2 → (empty)
/// 
/// Modifies string1 in place by replacing characters starting at index with string2.
/// This works because strings are now wrapped in Rc<RefCell<String>>.
fn putinterval(ctx: &mut Context) -> Result<(), String> {
    let source = ctx.pop().ok_or("Stack underflow".to_string())?;
    let index = ctx.pop().ok_or("Stack underflow".to_string())?;
    let dest = ctx.pop().ok_or("Stack underflow".to_string())?;
    
    match (dest, index, source) {
        (PostScriptValue::String(dest_str), PostScriptValue::Int(idx), PostScriptValue::String(src_str)) => {
            let idx = idx as usize;
            let src_borrowed = src_str.borrow();
            let mut dest_borrowed = dest_str.borrow_mut();
            
            // Check bounds
            if idx + src_borrowed.len() > dest_borrowed.len() {
                return Err("Range check error".to_string());
            }
            
            // Replace characters in dest starting at idx with characters from src
            // We need to work with byte indices for string slicing
            let mut dest_chars: Vec<char> = dest_borrowed.chars().collect();
            let src_chars: Vec<char> = src_borrowed.chars().collect();
            
            for (i, &ch) in src_chars.iter().enumerate() {
                dest_chars[idx + i] = ch;
            }
            
            *dest_borrowed = dest_chars.into_iter().collect();
            Ok(())
        }
        _ => Err("Type check error: putinterval expected string index string".to_string()),
    }
}

// ============================================================================
// Boolean and Bitwise Operations
// ============================================================================

/// eq: Test equality
/// Stack: any1 any2 → bool
fn eq(ctx: &mut Context) -> Result<(), String> {
    let b = ctx.pop().ok_or("Stack underflow".to_string())?;
    let a = ctx.pop().ok_or("Stack underflow".to_string())?;
    ctx.push(PostScriptValue::Bool(a == b));
    Ok(())
}

/// ne: Test inequality
/// Stack: any1 any2 → bool
fn ne(ctx: &mut Context) -> Result<(), String> {
    let b = ctx.pop().ok_or("Stack underflow".to_string())?;
    let a = ctx.pop().ok_or("Stack underflow".to_string())?;
    ctx.push(PostScriptValue::Bool(a != b));
    Ok(())
}

/// ge: Test greater than or equal
/// Stack: num1|string1 num2|string2 → bool
fn ge(ctx: &mut Context) -> Result<(), String> {
    let b = ctx.pop().ok_or("Stack underflow".to_string())?;
    let a = ctx.pop().ok_or("Stack underflow".to_string())?;
    match (a, b) {
        (PostScriptValue::Int(i1), PostScriptValue::Int(i2)) => ctx.push(PostScriptValue::Bool(i1 >= i2)),
        (PostScriptValue::Real(f1), PostScriptValue::Real(f2)) => ctx.push(PostScriptValue::Bool(f1 >= f2)),
        (PostScriptValue::Int(i1), PostScriptValue::Real(f2)) => ctx.push(PostScriptValue::Bool(i1 as f64 >= f2)),
        (PostScriptValue::Real(f1), PostScriptValue::Int(i2)) => ctx.push(PostScriptValue::Bool(f1 >= i2 as f64)),
        (PostScriptValue::String(s1), PostScriptValue::String(s2)) => ctx.push(PostScriptValue::Bool(*s1.borrow() >= *s2.borrow())),
        _ => return Err("Type check error".to_string()),
    }
    Ok(())
}

/// gt: Test greater than
/// Stack: num1|string1 num2|string2 → bool
fn gt(ctx: &mut Context) -> Result<(), String> {
    let b = ctx.pop().ok_or("Stack underflow".to_string())?;
    let a = ctx.pop().ok_or("Stack underflow".to_string())?;
    match (a, b) {
        (PostScriptValue::Int(i1), PostScriptValue::Int(i2)) => ctx.push(PostScriptValue::Bool(i1 > i2)),
        (PostScriptValue::Real(f1), PostScriptValue::Real(f2)) => ctx.push(PostScriptValue::Bool(f1 > f2)),
        (PostScriptValue::Int(i1), PostScriptValue::Real(f2)) => ctx.push(PostScriptValue::Bool(i1 as f64 > f2)),
        (PostScriptValue::Real(f1), PostScriptValue::Int(i2)) => ctx.push(PostScriptValue::Bool(f1 > i2 as f64)),
        (PostScriptValue::String(s1), PostScriptValue::String(s2)) => ctx.push(PostScriptValue::Bool(*s1.borrow() > *s2.borrow())),
        _ => return Err("Type check error".to_string()),
    }
    Ok(())
}

/// le: Test less than or equal
/// Stack: num1|string1 num2|string2 → bool
fn le(ctx: &mut Context) -> Result<(), String> {
    let b = ctx.pop().ok_or("Stack underflow".to_string())?;
    let a = ctx.pop().ok_or("Stack underflow".to_string())?;
    match (a, b) {
        (PostScriptValue::Int(i1), PostScriptValue::Int(i2)) => ctx.push(PostScriptValue::Bool(i1 <= i2)),
        (PostScriptValue::Real(f1), PostScriptValue::Real(f2)) => ctx.push(PostScriptValue::Bool(f1 <= f2)),
        (PostScriptValue::Int(i1), PostScriptValue::Real(f2)) => ctx.push(PostScriptValue::Bool(i1 as f64 <= f2)),
        (PostScriptValue::Real(f1), PostScriptValue::Int(i2)) => ctx.push(PostScriptValue::Bool(f1 <= i2 as f64)),
        (PostScriptValue::String(s1), PostScriptValue::String(s2)) => ctx.push(PostScriptValue::Bool(*s1.borrow() <= *s2.borrow())),
        _ => return Err("Type check error".to_string()),
    }
    Ok(())
}

/// lt: Test less than
/// Stack: num1|string1 num2|string2 → bool
fn lt(ctx: &mut Context) -> Result<(), String> {
    let b = ctx.pop().ok_or("Stack underflow".to_string())?;
    let a = ctx.pop().ok_or("Stack underflow".to_string())?;
    match (a, b) {
        (PostScriptValue::Int(i1), PostScriptValue::Int(i2)) => ctx.push(PostScriptValue::Bool(i1 < i2)),
        (PostScriptValue::Real(f1), PostScriptValue::Real(f2)) => ctx.push(PostScriptValue::Bool(f1 < f2)),
        (PostScriptValue::Int(i1), PostScriptValue::Real(f2)) => ctx.push(PostScriptValue::Bool((i1 as f64) < f2)),
        (PostScriptValue::Real(f1), PostScriptValue::Int(i2)) => ctx.push(PostScriptValue::Bool(f1 < i2 as f64)),
        (PostScriptValue::String(s1), PostScriptValue::String(s2)) => ctx.push(PostScriptValue::Bool(*s1.borrow() < *s2.borrow())),
        _ => return Err("Type check error".to_string()),
    }
    Ok(())
}

/// and: Logical or bitwise AND
/// Stack: bool1|int1 bool2|int2 → bool|int
fn and(ctx: &mut Context) -> Result<(), String> {
    let b = ctx.pop().ok_or("Stack underflow".to_string())?;
    let a = ctx.pop().ok_or("Stack underflow".to_string())?;
    match (a, b) {
        (PostScriptValue::Bool(b1), PostScriptValue::Bool(b2)) => ctx.push(PostScriptValue::Bool(b1 && b2)),
        (PostScriptValue::Int(i1), PostScriptValue::Int(i2)) => ctx.push(PostScriptValue::Int(i1 & i2)),
        _ => return Err("Type check error".to_string()),
    }
    Ok(())
}

/// not: Logical or bitwise NOT
/// Stack: bool|int → bool|int
fn not(ctx: &mut Context) -> Result<(), String> {
    let a = ctx.pop().ok_or("Stack underflow".to_string())?;
    match a {
        PostScriptValue::Bool(b) => ctx.push(PostScriptValue::Bool(!b)),
        PostScriptValue::Int(i) => ctx.push(PostScriptValue::Int(!i)),
        _ => return Err("Type check error".to_string()),
    }
    Ok(())
}

/// or: Logical or bitwise OR
/// Stack: bool1|int1 bool2|int2 → bool|int
fn or(ctx: &mut Context) -> Result<(), String> {
    let b = ctx.pop().ok_or("Stack underflow".to_string())?;
    let a = ctx.pop().ok_or("Stack underflow".to_string())?;
    match (a, b) {
        (PostScriptValue::Bool(b1), PostScriptValue::Bool(b2)) => ctx.push(PostScriptValue::Bool(b1 || b2)),
        (PostScriptValue::Int(i1), PostScriptValue::Int(i2)) => ctx.push(PostScriptValue::Int(i1 | i2)),
        _ => return Err("Type check error".to_string()),
    }
    Ok(())
}

// ============================================================================
// Flow Control
// ============================================================================

/// if: Conditional execution
/// Stack: bool proc → (empty)
/// Executes proc if bool is true
fn if_op(ctx: &mut Context) -> Result<(), String> {
    let proc = ctx.pop().ok_or("Stack underflow".to_string())?;
    let bool_val = ctx.pop().ok_or("Stack underflow".to_string())?;
    match bool_val {
        PostScriptValue::Bool(true) => {
            // Execute the procedure by pushing it to the execution stack
            match proc {
                PostScriptValue::Block(block) => {
                    for item in block.iter().rev() {
                        ctx.execution_stack.push(item.clone());
                    }
                }
                _ => ctx.execution_stack.push(proc),
            }
        }
        PostScriptValue::Bool(false) => {}
        _ => return Err("Type check error: if expected bool".to_string()),
    }
    Ok(())
}

/// ifelse: Conditional branching
/// Stack: bool proc1 proc2 → (empty)
/// Executes proc1 if bool is true, proc2 if false
fn ifelse(ctx: &mut Context) -> Result<(), String> {
    let proc2 = ctx.pop().ok_or("Stack underflow".to_string())?;
    let proc1 = ctx.pop().ok_or("Stack underflow".to_string())?;
    let bool_val = ctx.pop().ok_or("Stack underflow".to_string())?;
    match bool_val {
        PostScriptValue::Bool(true) => {
            match proc1 {
                PostScriptValue::Block(block) => {
                    for item in block.iter().rev() {
                        ctx.execution_stack.push(item.clone());
                    }
                }
                _ => ctx.execution_stack.push(proc1),
            }
        }
        PostScriptValue::Bool(false) => {
            match proc2 {
                PostScriptValue::Block(block) => {
                    for item in block.iter().rev() {
                        ctx.execution_stack.push(item.clone());
                    }
                }
                _ => ctx.execution_stack.push(proc2),
            }
        }
        _ => return Err("Type check error: ifelse expected bool".to_string()),
    }
    Ok(())
}

/// for: Loop with start, step, and limit
/// Stack: initial step limit proc → (empty)
/// Executes proc for each value from initial to limit, incrementing by step
/// The current loop value is pushed onto the stack before each execution of proc
fn for_op(ctx: &mut Context) -> Result<(), String> {
    let proc = ctx.pop().ok_or("Stack underflow".to_string())?;
    let limit = ctx.pop().ok_or("Stack underflow".to_string())?;
    let step = ctx.pop().ok_or("Stack underflow".to_string())?;
    let initial = ctx.pop().ok_or("Stack underflow".to_string())?;
    
    // Convert all values to f64 for consistent handling
    let (current, step_val, limit_val) = match (initial, step, limit) {
        (PostScriptValue::Int(i), PostScriptValue::Int(s), PostScriptValue::Int(l)) => (i as f64, s as f64, l as f64),
        (PostScriptValue::Real(i), PostScriptValue::Real(s), PostScriptValue::Real(l)) => (i, s, l),
        (i, s, l) => {
            let i = match i { PostScriptValue::Int(v) => v as f64, PostScriptValue::Real(v) => v, _ => return Err("Type error".to_string()) };
            let s = match s { PostScriptValue::Int(v) => v as f64, PostScriptValue::Real(v) => v, _ => return Err("Type error".to_string()) };
            let l = match l { PostScriptValue::Int(v) => v as f64, PostScriptValue::Real(v) => v, _ => return Err("Type error".to_string()) };
            (i, s, l)
        }
    };

    // Push ForLoop state to execution stack - the interpreter will handle the iteration
    ctx.execution_stack.push(PostScriptValue::ForLoop {
        current,
        step: step_val,
        limit: limit_val,
        proc: Box::new(proc),
    });
    Ok(())
}

/// repeat: Execute a procedure n times
/// Stack: n proc → (empty)
fn repeat(ctx: &mut Context) -> Result<(), String> {
    let proc = ctx.pop().ok_or("Stack underflow".to_string())?;
    let count = ctx.pop().ok_or("Stack underflow".to_string())?;
    
    match count {
        PostScriptValue::Int(n) => {
            if n < 0 {
                return Err("Range check error".to_string());
            }
            // Push RepeatLoop state to execution stack - the interpreter will handle the iteration
            ctx.execution_stack.push(PostScriptValue::RepeatLoop {
                count: n,
                proc: Box::new(proc),
            });
        }
        _ => return Err("Type check error".to_string()),
    }
    Ok(())
}

/// quit: Terminate the interpreter
/// Stack: (empty) → (exits program)
fn quit(_ctx: &mut Context) -> Result<(), String> {
    std::process::exit(0);
}

// ============================================================================
// Input/Output Operations
// ============================================================================

/// print: Print a string to stdout
/// Stack: string → (empty)
fn print(ctx: &mut Context) -> Result<(), String> {
    let s = ctx.pop().ok_or("Stack underflow".to_string())?;
    match s {
        PostScriptValue::String(s) => print!("{}", s.borrow()),
        _ => return Err("Type check error".to_string()),
    }
    Ok(())
}

/// =: Print text representation of a value
/// Stack: any → (empty)
/// Prints the value in human-readable form
fn eq_print(ctx: &mut Context) -> Result<(), String> {
    let any = ctx.pop().ok_or("Stack underflow".to_string())?;
    println!("{}", any);
    Ok(())
}

/// ==: Print PostScript representation of a value
/// Stack: any → (empty)
/// Prints the value in PostScript syntax (e.g., strings with parentheses)
fn eqeq_print(ctx: &mut Context) -> Result<(), String> {
    let any = ctx.pop().ok_or("Stack underflow".to_string())?;
    println!("{}", any);
    Ok(())
}
