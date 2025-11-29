# PostScript Interpreter in Rust

A PostScript interpreter implemented in Rust, supporting a comprehensive subset of PostScript commands with both dynamic (default) and lexical scoping.

## Overview

This interpreter implements robust support for:
- Stack-based operations
- Arithmetic and mathematical functions
- Dictionary management
- String manipulation
- Boolean and bitwise operations
- Flow control structures
- Input/output operations

### Why Rust?

Rust was chosen for this implementation due to its unique combination of:
- **Memory safety** without garbage collection
- **Performance** comparable to C/C++
- **Strong type system** with pattern matching
- **Excellent error handling** with the `Result` type
- **Modern tooling** (Cargo, Clippy, Rustfmt)

## Building and Running

### Prerequisites
- Rust (latest stable version)
- Cargo (comes with Rust)

### Build
```bash
# Development build
cargo build

# Optimized release build
cargo build --release
```

### Run

**Interactive REPL:**
```bash
cargo run
```

**Execute a PostScript file:**
```bash
cargo run -- script.ps
```

**With lexical scoping:**
```bash
cargo run -- --lexical script.ps
```

### Scoping Modes

The interpreter supports both scoping models:

- **Dynamic Scoping (default)**: Variables are resolved in the calling context, following standard PostScript behavior
- **Lexical Scoping**: Variables are resolved in the defining context, enabled with the `--lexical` flag

Example:
```bash
# Dynamic scoping (default)
cargo run -- scoping_test.ps

# Lexical scoping
cargo run -- --lexical scoping_test.ps
```

## Supported Commands (47/47) ✅

### Stack Manipulation (6/6)
- `exch` - Exchange top two stack items
- `pop` - Remove top item from stack
- `copy` - Copy top n items on stack
- `dup` - Duplicate top stack item
- `clear` - Clear entire operand stack
- `count` - Count items on stack

### Arithmetic Operations (12/12)
- `add` - Addition (supports int and real)
- `sub` - Subtraction
- `mul` - Multiplication
- `div` - Division (returns real)
- `idiv` - Integer division
- `mod` - Modulo operation
- `abs` - Absolute value
- `neg` - Negation
- `ceiling` - Round up to nearest integer
- `floor` - Round down to nearest integer
- `round` - Round to nearest integer
- `sqrt` - Square root

### Dictionary Operations
- `dict` - Create dictionary with specified capacity
- `length` - Get number of key-value pairs
- `maxlength` - Get dictionary capacity
- `begin` - Push dictionary onto dictionary stack
- `end` - Pop dictionary stack
- `def` - Define key-value pair in current dictionary

### String Operations (4/4)
- `length` - Get string length
- `get` - Get character at index (returns ASCII value)
- `getinterval` - Extract substring
- `putinterval` - Replace part of string (in-place mutation)

### Boolean and Bitwise Operations (11/11)
- `eq` - Test equality
- `ne` - Test inequality
- `ge` - Greater than or equal
- `gt` - Greater than
- `le` - Less than or equal
- `lt` - Less than
- `and` - Logical/bitwise AND
- `or` - Logical/bitwise OR
- `not` - Logical/bitwise NOT
- `true` - Boolean constant
- `false` - Boolean constant

### Flow Control (5/5)
- `if` - Conditional execution
- `ifelse` - Conditional branching
- `for` - Loop with start, step, and limit
- `repeat` - Repeat procedure n times
- `quit` - Terminate interpreter

### Input/Output (3/3)
- `print` - Print string to stdout
- `=` - Print text representation of value
- `==` - Print PostScript representation of value

## Testing

### Run Test Scripts

```bash
# Basic functionality test
cargo run -- test.ps

# Scoping tests (dynamic)
cargo run -- scoping_test.ps

# Scoping tests (lexical)
cargo run -- --lexical scoping_test.ps

# Comprehensive command verification
cargo run -- comprehensive_test.ps

# String mutation tests
cargo run -- string_mutation_test.ps
```

### Verification

A comprehensive test suite (`comprehensive_test.ps`) verifies all implemented commands. See the verification report for detailed test results and coverage analysis.

## Implementation Details

### String Mutation

Strings in this interpreter use `Rc<RefCell<String>>` to support mutable shared references, matching PostScript's string semantics. This means:

- Strings can be modified in place with `putinterval`
- Multiple references to the same string share the underlying data
- Mutations are visible through all references

**Example:**
```postscript
(hello world) dup        % Create two references to same string
0 (HELLO) putinterval    % Modify via first reference
=                         % Both show: (HELLO world)
=
```

## Project Structure

```
postscript-interpreter/
├── src/
│   ├── lib.rs              # Library entry point
│   ├── main.rs             # CLI entry point
│   ├── types.rs            # PostScript value types and context
│   ├── parser.rs           # PostScript parser
│   ├── interpreter.rs      # Interpreter execution engine
│   └── commands.rs         # Built-in command implementations
├── test.ps                 # Basic test script
├── scoping_test.ps         # Scoping behavior tests
├── comprehensive_test.ps   # Full command verification
├── Cargo.toml              # Rust project configuration
└── README.md               # This file
```

## Examples

### Basic Arithmetic
```postscript
3 4 add =          % Prints: 7
10 3 idiv =        % Prints: 3
10 3 mod =         % Prints: 1
16 sqrt =          % Prints: 4.0
```

### Stack Operations
```postscript
1 2 3 count =      % Prints: 3
1 2 exch = =       % Prints: 1, then 2
5 dup = =          % Prints: 5, then 5
```

### Flow Control
```postscript
% Conditional
10 5 gt { (Greater) = } { (Not greater) = } ifelse

% Loop
1 1 5 { = } for    % Prints: 1 2 3 4 5

% Repeat
3 { (Hello) print } repeat
```

### Dictionaries
```postscript
10 dict begin
/x 42 def
/y 100 def
x y add =          % Prints: 142
end
```

## Implementation Details

- **Language:** Rust (2021 edition)
- **Architecture:** Stack-based interpreter with execution stack
- **Scoping:** Configurable dynamic or lexical scoping
- **Error Handling:** Comprehensive error messages for stack underflow, type errors, and range errors
- **Type System:** Strong typing with support for integers, reals, strings, arrays, dictionaries, booleans, and procedures

## Performance

Built with Rust's zero-cost abstractions and LLVM optimization, the interpreter delivers:
- Native code execution speed
- Minimal memory overhead
- Predictable performance without garbage collection pauses
