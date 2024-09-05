# Application Notes

## Expected Document Layouts

Applications using Datum are generally expected to stick to one of five "document layouts".

These layouts may also be embedded in lists.

### List

The document is a list (for example, a list of allowed users, or directory patterns).

No start/end list tokens are used for this outer list, as it is not necessary.

```scheme
; List of files and directories to exclude from export
".git"
".classpath"
```

The list may contain more complex values:

```scheme
; Each excluded entity must have an associated reason
(".git" "metadata, history")
(".classpath" "ide")
```

The list may be a sequence or stream of commands -- see Typical Representations.

### Map

The document is a stream of key/value pairs. The pairs are not explicitly marked with `()`, though if the values are complex then they should be lists (or maps-as-lists in the same style, or maps-as-lists with an identifying symbol).

This model can be very useful for configuration files.

```scheme
ignore (
	".git"
	".classpath"
)
exceptions (
	".git/HEAD"
)
```

### Single Value

The document represents a single Datum value. Further values are disallowed. Chances are high that the value could probably be in one of the other formats, or embedded in another file, but for whatever reason, it isn't.

This document format is not recommended. Chances are pretty good that if you think you want this format, you want List or Map format instead.

### Arbitrary

The document is grammatically correct Datum, but there are out-of-band factors imposing layers of additional grammar.

It's a bit of a spectrum; Map could be considered a 'sub-format' of this, but Map attempts to impose some strict rules to keep things sensible.

Prefixed could be considered a 'sub-format' of this, but has its roots in LISP.

To be clear, there are good uses for this document format, but Map is as far as I recommend anyone goes.

### Prefixed

Datum's design borrows heavily from LISPs, and LISP concepts can fit well in Datum. Prefixed is one of these sorts of things.

In short: Take any other format, and add a symbol at the start indicating what the thing is.

```scheme
ignore-list
".git"
".classpath"
```

In LISPs, this is only really seen as a value, i.e. `(+ 2 2)`, but the idea is the same.

## Serde Mapping

The Serde mapping defines how Datum interacts with the [serde](https://serde.rs/) Rust crate.

In practice, I expect this to allow quick and immediate use of Datum for configuration files.

**This is still in flux until the implementation stabilizes.**

In particular, it'd be nice if newtypes always acted as they do in `RootDeserializer` (saving redundant `()`), but the implementation needs to be worked out before this can be written down here.

### Deserialization

#### Deserializers

There are two deserializers.

* `PlainDeserializer`: Standard deserializer.
* `RootDeserializer`: Intended for 'the whole document' values. Removes `()` from a lot of elements.

The 'plain' format is an as-is 1:1 Serde deserializer, where repeated calls deserialize further values in the file. EOF detection requires calling a custom function to read ahead to determine if EOF has been reached.

The root format will be described later.

#### 'Any'

This covers the generic 'dynamically typed' case, and essentially produces a Datum AST in Serde format.

* Strings and symbols become strings.
* Integers become `i64` and floats become `f64`.
* Booleans become `bool`.
* Lists become sequences.
* Nil becomes `unit`.

These deserialization types are treated as equivalent to `deserialize_any`:

* The simple number/primitive types: `bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char`
* Strings: `str string identifier`
* Anything that would end up a `seq`: `seq tuple tuple_struct`
* This: `ignored_any`
* Unsupported things: `bytes byte_buf` (these would require `Vec` to support; Serde has no streaming byte buffer deserialization).

#### Enum

If the next token is a symbol, that symbol is treated as a unit variant.

If it is a list start, then the syntax `(variant ...)` is expected.

If neither a symbol or a list start is found, the value is parsed as per `any`.

#### Option

If the next token is `#nil`, then `None` is assumed. Otherwise, `Some` is assumed, and the token is held for parsing the value within.

#### Unit

A list start is checked for. If it is found, a list end is expected, and that's the unit.

If a list start is not found, then the value is parsed as per `any`.

`unit_struct` is treated equivalently to `unit`.

It is important that `#nil` and `()` both be valid ways of writing `unit`; the former works in `any` contexts and the latter is valid in Options.

#### Map

A list start is checked for. If it is found, a map is visited as per the document layout description above.

If a list start is not found, then the value is parsed as per `any`.

`struct` is treated equivalently to `map`.

#### Newtype Struct

`newtype_struct` is simply visited without anything actually parsed by the handler.

#### 'Root' Deserializer

The 'root' deserializer wraps the 'plain' deserializer, and works as follows:

* `deserialize_any` is simply passed to the underlying `PlainDeserializer`.
	* This extends the problems `deserialize_any` has with maps and enums to sequences as well. This is expected.
* `deserialize_newtype_struct` has the contents run through `RootDeserializer` (same implementation idea as `PlainDeserializer` but it's important we stay in the root format for this case).
* `deserialize_option` checks for EOF. If EOF is found, it takes the `visit_none` branch. Otherwise, it always takes the `visit_some` branch due to ambiguity (`(#nil b)` is rendered as `#nil b` here) and reruns through `RootDeserializer`.
	* This is most likely to find its use in the "tree bark" file versioning pattern, where a file is built up of layers of additional data for each version of the format. Datum's not really expected to be used this way, but if someone _does_ do this, it's supported.
* `deserialize_enum` uses a special 'unwrapped' (`()`-less) version of the `(variant ...)` format.
* Anything that `PlainDeserializer` would attempt to deserialize as a map assumes an 'unwrapped' map, like with enums.
* `deserialize_seq`, `deserialize_tuple`, and `deserialize_tuple_struct` all assume an 'unwrapped' seq.
* Values _inside_ a root-level collection, i.e. keys/values of maps, values in seqs, etcetc. are parsed by `PlainDeserializer`. Options are not collections.
	* Enum variants and the individual values in an enum variant are considered 'inside a collection'.
		* An exception to this is specifically newtype variants. Their value (but not their identifying key) remains at root level.
	* In regards to tuples, Serde performs an 'early exit' of a seq once all elements of the tuple are deserialized. It is expected behaviour that a value can follow the tuple.
	* `#nil` as `Option<Vec<Option<bool>>>` becomes `Some(vec![None])`.
	* `1 2` as `(i32, i32)` becomes `(1, 2)`.
	* `1 (2 3)` as `(i32, (i32, i32))` becomes `(1, (2, 3))`.
	* `1 2 3 4` as `(i32, i32)` followed by `(i32, i32)` becomes `(1, 2)` followed by `(3, 4)`.

To interpret what 'unwrapped' means here, assume any relevant check for list start (specifically: Enum, Map, lists) automatically succeeds, and EOF is the list end token.

This deserializer is intended for circumstances where the user wishes to serialize/deserialize an entire document rather than a stream of values.

### Serialization

Serialization has been handled with reference to the Serde implementations of `Deserialize`, to ensure that they deserialize cleanly.

* The bool/i/u/f types are trivial and don't need describing in detail, except that i128/u128 are not supported, and u64 is converted to i64.
* `serialize_char` is implemented as `self.collect_str(&v)`, producing the character as a string, which Serde can read.
* Unit structs are serialized as units.
* Newtype structs are serialized as their interiors.
* `None` is `#nil` and `Some` is pass-through. Correspondingly, `()` is `()` (as `#nil` would be ambiguous).
* Enum unit variants are written as symbols, newtype variants are written as `(variant value)`, tuple variants are written as `(variant value...)`, struct variants are `(variant key value...)`.
* Structs are written like maps, but the keys are written as symbols.
* Sequences, tuples, and tuple structs are just lists.
* Maps are lists where the contents are as per the document layout description above.
* Strings are strings. That's all.
* Byte arrays cannot be serialized at present.

For `RootSerializer`, the rules change somewhat:

* The bool/i/u/f types, along with strings, units, and unit structs, are forwarded as-is to `PlainSerializer`.
* `None` is not supported because in this form, `None` and `Some(vec![])` are ambiguous. `Some` is supported by simply serializing whatever's inside.
	* _As long as the value is at the end of the file,_ the user can 'write' `None` by simply not writing anything.
* Enums are always forwarded with essentially an "unwrapped `()` form", i.e. `(variant value)` becomes `variant value`.
	* _Newtype variants in particular_ have their value written as a root-level element. Tuple variants and struct variants are treated like tuples and structs respectively.
* Structs, maps, sequences, tuples, and tuple structs all become their "unwrapped `()` forms".
* A key difference in formatting is that `PlainSerializer` attempts to indent seqs, maps, and structs (and struct variants), but not tuples (or tuple variants, or newtype variants, etc.). `RootSerializer` _always_ adds newlines between each root-level element.
* Sequence/tuple elements, struct/map keys/values, etc. are passed to `PlainSerializer`; the rule of thumb is that `RootSerializer` removes the outermost `()` pair from a value.

_A key implication here is that for `RootSerializer`, the end of the value is delimited by the end of the file._
