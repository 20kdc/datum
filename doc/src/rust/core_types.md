# Core Datatypes

`DatumError`, `DatumResult`, and `DatumPipe` make up the core types used across the library.

## DatumError

`DatumError` is the error type for all errors produced by the `datum` crate.

It is paired with the `datum_error!(kind, message)` macro, which is divided into a machine-friendly broad type (which should be version-stable) and a human-friendly fine type (not so version-stable).

`DatumResult` is the inevitable result type.

## DatumPipe

`DatumPipe` is `datum`'s way of providing a "push API".

Compared to "pull APIs" such as `Iterator`, push APIs are more flexible around async IO without actually being async.

For instance, consider the following parser:

```rust
# extern crate datum;
# use datum::{DatumPipe, DatumResult, DatumOffset};

struct MyExampleParser(u8);

impl DatumPipe for MyExampleParser {
	type Input = char;
	type Output = u8;

	fn feed<F: FnMut(DatumOffset, u8) -> DatumResult<()>>(&mut self, at: DatumOffset, c: Option<char>, f: &mut F) -> DatumResult<()> {
		if let None = c {
			return Ok(())
		}
		let c = c.unwrap();
		if c == '+' {
			self.0 = self.0.wrapping_add(1);
			Ok(())
		} else if c == '-' {
			self.0 = self.0.wrapping_sub(1);
			Ok(())
		} else if c == '.' {
			f(at, self.0)
		} else if c == '!' {
			self.0 = self.0.wrapping_add(32);
			Ok(())
		} else if c == ':' {
			f(at, self.0)?;
			f(at, self.0)
		} else {
			// we just ignore unknown characters
			Ok(())
		}
	}
}

let mut test = MyExampleParser(0);

// we got some initial bytes...
test.feed(0, Some('!'), &mut |_, _| Ok(())).unwrap();
test.feed(0, Some('!'), &mut |_, _| Ok(())).unwrap();
test.feed(0, Some('+'), &mut |_, _| Ok(())).unwrap();
// (network socket ran out of data/etc...)
// (...time passes...)
// ...we got more data!
test.feed(0, Some('.'), &mut |_, _| Ok(())).unwrap();
test.feed(0, Some('+'), &mut |_, _| Ok(())).unwrap();
test.feed(0, Some('.'), &mut |_, _| Ok(())).unwrap();
test.feed(0, Some('+'), &mut |_, _| Ok(())).unwrap();
test.feed(0, Some('.'), &mut |_, _| Ok(())).unwrap();
// Socket closed.
test.feed(0, None, &mut |_, _| Ok(())).unwrap();
```

Now, this parser is pretty absurd, but it goes over the basic principles of the `DatumPipe` API:

* Implementations can input and return whatever types they like, though they always use `DatumError`.
* Semi-opaque `u64` 'offsets' are passed through (useful for error handling).
	* Implementations should pick the offset that is most convenient for reuse in various circumstances, including zero-copy parsing. As a result, offsets may "jump back" to the start of a relevant object; tokenizers do this.
* Implementations may return multiple results from a single call using the closure provided.
* The closure can itself return errors.
* EOF is represented as a final `None` value.

The main advantage here comes from the ability to asynchronously handle data without using async.

Like iterators, `DatumPipe` can be chained; this is the use of `DatumPipe::compose`.

For uses where blocking IO is fine, `Iterator::via_datum_pipe` connects a `DatumPipe` into an `Iterator` chain.

This works nicely for simple recursive descent parsing, though with the problems of simple recursive descent parsing.

Finally, if `alloc` is present, there are a number of pre-composed chains:

* `datum_byte_to_token_pipeline`: `u8` stream to `DatumToken<String>` stream.
* `datum_char_to_token_pipeline`: `char` stream to `DatumToken<String>` stream.
* `datum_byte_to_value_pipeline`: `u8` stream to `DatumValue` stream.
* `datum_char_to_value_pipeline`: `char` stream to `DatumValue` stream.

Chances are, for any simple reading task, it will be enough to take the results of, say, [read_to_string](https://doc.rust-lang.org/std/fs/fn.read_to_string.html), run `.chars().via_datum_pipe(...)` and get what you want.

In fact, let's do something like that:

```rust
# extern crate datum;
use datum::{IntoViaDatumPipe, datum_char_to_value_pipeline};
let source = "(the quick brown fox) jumped (over (the lazy dog))";
for v in source.chars().via_datum_pipe(datum_char_to_value_pipeline()).map(|v| v.expect("the input should be valid")) {
	println!("{}", v);
}
```

## Non-Allocating Iterators: `DatumBoundedPipe`

So a problem with `DatumViaPipe` is that it works by using a `VecDeque` to store the queued elements.

However, in no-std environments, `VecDeque` isn't a thing.

Due to various limitations -- MSRV limitations, const generics limitations... point is, due to various limitations, the solution for this is a bit internally awkward, but should be fine as a library user -- as far as you should be concerned, you should be able to pass any bounded pipe or composed chain thereof to `via_datum_buf_pipe` and it just works.

There are two key traits which are the 'entrypoint' to non-allocating iterator pipelines in Datum.

* `DatumBoundedQueue<V>` is a trait defined on `()` and `Option<(V, Q)>` (where `Q: DatumBoundedQueue<V>`). It describes fixed-size queue implementations, and implements various mathematical operations which are used for composition.
* `DatumBoundedPipe` is a trait defined on a `DatumPipe`. It provides `type OutputQueue: DatumBoundedQueue<(DatumOffset, Self::Output)>`, which in practice defines a type you can use as an output queue for the output of a `feed` call to the given pipe.

`ViaDatumBufPipe` connects the resulting pipelines to iterators.

### Why not an iterator stack?

The pipeline couldn't be represented as an iterator stack without either `impl Trait` or GATs.

The problem was basically the difference between "a compose is processing this" and "a bounded pipe node is processing this".

And since composes are made up of arbitrary other pipes, this was kinda important.

The conversion for one bounded pipe would be, say: `PipeIterator<I, P>`.

The conversion for a compose of two bounded pipes would be, say: `PipeIterator<PipeIterator<I, A>, B>`.

The problem here is that `I` type argument, the base iterator. A compose can have (and does have) an associated type covering the composition. But adding an iterator into that, say, `type AsIterator<I: Iterator<...>> = ...;` -- that makes the associated type generic, a GAT.

The other option, the first one I tried, was simply using `impl Trait`. After making it all work, I then found out `impl Trait` was not actually supported on the target MSRV.
