# PostScript Interpreter in Rust

A PostScript interpreter supporting a subset of commands, with support for both dynamic (default) and lexical scoping.

## Building and Running

### Prerequisites
- Rust (latest stable)

### Build
```bash
cargo build --release
```

### Run
Run interactively (REPL):
```bash
cargo run
```

Run a script:
```bash
cargo run -- script.ps
```

### Scoping
The interpreter uses **dynamic scoping** by default, as per standard PostScript.
To enable **lexical scoping**, use the `--lexical` flag:

```bash
cargo run -- --lexical script.ps
```

## Supported Commands

### Stack Manipulation
`exch`, `pop`, `copy`, `dup`, `clear`, `count`

### Arithmetic
`add`, `sub`, `mul`, `div`, `idiv`, `mod`, `abs`, `neg`, `ceiling`, `floor`, `round`, `sqrt`

### Dictionary
`dict`, `length`, `maxlength`, `begin`, `end`, `def`

### String
`length`, `get`, `getinterval`, `putinterval`

### Boolean/Bit
`eq`, `ne`, `ge`, `gt`, `le`, `lt`, `and`, `not`, `or`, `true`, `false`

### Flow Control
`if`, `ifelse`, `for`, `repeat`, `quit`

### I/O
`print`, `=`, `==`

## Testing
Run the provided test scripts:
```bash
cargo run -- test.ps
cargo run -- scoping_test.ps
cargo run -- --lexical scoping_test.ps
```
