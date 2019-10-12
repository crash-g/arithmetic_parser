# Simple parser and evaluator for arithmetic expressions

## Usage
```rust
use arithmetic_parser as parser;
let expression = parser::ArithmeticExpression::parse("(x+y)/(x-y)").unwrap();
let variables = [("x", 5_f64), ("y", 1_f64)].iter().cloned().collect();
assert_eq!(1.5, expression.evaluate(&variables).unwrap());

```

## Compilation

    cargo build --release

## Run interactive environment

    cargo run --example interactive

## Foreign Function Interface (FFI)

A basic `C` API is provided as a separate crate. See
[arithmetic_parser_wrapper](ffi/README.md) for more information.
