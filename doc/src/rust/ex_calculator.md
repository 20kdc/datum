# Example: Desk Calculator

This example goes over a 'desk calculator', which executes expressions on floating-point numbers.

First stop, the `main` function:

```rust,ignore
{{#include ../../../examples/rust/calculator/src/main.rs:main}}
```

This function's concern is dealing with parsing errors and providing the 'UI', but it does show how to quickly get data in from Datum into values.

All of the actual Datum-related work here happens in `combo_buffer.chars().via_datum_pipe(datum_rs::datum_char_to_value_pipeline())`.

This converts an iterator of characters to an iterator of Datum values, with possible errors.

The rest of the code manages a buffer (for incomplete lists) and `rustyline` so that the result is a usable interface.

Meanwhile, the compiler takes those values and turns them into expressions:

```rust,ignore
{{#include ../../../examples/rust/calculator/src/main.rs:compiler}}
```

Now that we've gone over the real end-to-end core of the calculator, let's look at the rest.

The actual expression format isn't particularly flexible, but it works:

```rust,ignore
{{#include ../../../examples/rust/calculator/src/main.rs:virtual-machine}}
```

These compiled expressions, when put into functions, have to be stored somewhere.

This also contains the 'default functions' so that basics like addition and subtraction don't have to be explicitly added into the compiler.

```rust,ignore
{{#include ../../../examples/rust/calculator/src/main.rs:environment}}
```

Finally, the `execute` function, referred to but deliberately skipped earlier, is responsible for the "meta-syntax" that allows defining functions.

While the result would still be a calculator without it, it would be less useful.

```rust,ignore
{{#include ../../../examples/rust/calculator/src/main.rs:executor}}
```
