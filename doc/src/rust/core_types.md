# Core Datatypes

`DatumError`, `DatumResult`, and `DatumPipe` make up the core types used across the library.

## DatumError

`DatumError` is the error type for all errors produced by `datum_rs`.

It is paired with the `datum_error!(kind, message)` macro, which is divided into a machine-friendly broad type (which should be version-stable) and a human-friendly fine type (not so version-stable).

`DatumResult` is the inevitable result type.

## DatumPipe

`DatumPipe` is `datum_rs`'s way of providing a "push API".

Compared to "pull APIs" such as `Iterator`, push APIs are more flexible around async IO without actually being async.

For instance, consider the following parser:

```rust
# extern crate datum_rs;
# use datum_rs::{DatumPipe, DatumResult};

struct MyExampleParser(u8);

impl DatumPipe for MyExampleParser {
	type Input = char;
	type Output = u8;

	fn feed<F: FnMut(u8) -> DatumResult<()>>(&mut self, c: char, f: &mut F) -> DatumResult<()> {
		if c == '+' {
			self.0 = self.0.wrapping_add(1);
			Ok(())
		} else if c == '-' {
			self.0 = self.0.wrapping_sub(1);
			Ok(())
		} else if c == '.' {
			f(self.0)
		} else if c == '!' {
			self.0 = self.0.wrapping_add(32);
			Ok(())
		} else if c == ':' {
			f(self.0)?;
			f(self.0)
		} else {
			// we just ignore unknown characters
			Ok(())
		}
	}

	fn eof<F: FnMut(u8) -> DatumResult<()>>(&mut self, f: &mut F) -> DatumResult<()> {
		// no issues with interruption in this language
		Ok(())
	}
}

let mut test = MyExampleParser(0);

// we got some initial bytes...
test.feed('!', &mut |_| Ok(())).unwrap();
test.feed('!', &mut |_| Ok(())).unwrap();
test.feed('+', &mut |_| Ok(())).unwrap();
// (network socket ran out of data/etc...)
// (...time passes...)
// ...we got more data!
test.feed('.', &mut |_| Ok(())).unwrap();
test.feed('+', &mut |_| Ok(())).unwrap();
test.feed('.', &mut |_| Ok(())).unwrap();
test.feed('+', &mut |_| Ok(())).unwrap();
test.feed('.', &mut |_| Ok(())).unwrap();
```

Now, this parser is pretty absurd, but it goes over the basic principles of the `DatumPipe` API:

* Implementations can input and return whatever types they like, though they always use `DatumError`.
* Implementations may return multiple results from a single call using the closure provided.
* The closure can itself return errors.
* Implementations may catch stream interruption using the `eof` handler.

The main advantage here comes from the ability to asynchronously handle data without using async.

Like iterators, `DatumPipe` can be chained; this is the use of `DatumPipe::compose`.

For uses where blocking IO is fine, `Iterator::via_datum_pipe` connects a `DatumPipe` into an `Iterator` chain.

This works nicely for simple recursive descent parsing, though with the problems of simple recursive descent parsing.

Finally, if `alloc` is present, there are a number of pre-composed chains:

* `datum_byte_to_token_pipeline`: `u8` stream to `DatumToken<String>` stream.
* `datum_char_to_token_pipeline`: `char` stream to `DatumToken<String>` stream.
* `datum_byte_to_value_pipeline`: `u8` stream to `DatumValue` stream.
* `datum_char_to_value_pipeline`: `char` stream to `DatumValue` stream.

Chances are, for any simple reading task, it will be enough to take the results of, say, `read_to_string`, run `.chars().via_datum_pipe(...)` and get what you want.

## AsyncUnwrap

_Due to the `noop_waker` feature requiring Nightly Rust, this functionality requires the `unsafe` feature to reimplement the required functionality. This requirement may be dropped at a later date._

`AsyncUnwrap` is a trait that allows Datum code to be written in Async Rust without sacrificing the ability to easily use it from runtimeless, non-async Rust.

It does this by providing a very tiny async executor, callable as a trait method on `Future`.

If the async code stops for any reason, it panics.

```rust
# extern crate datum_rs;
use datum_rs::AsyncUnwrap;

println!("{}", async {
	println!("Blocking IO is fine.");
	1
}.expect("No async IO."));
```
