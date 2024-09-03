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

### Root Deserializers

There are three root deserializers. One is the 'standard' deserializer, and the other two affect the mapping significantly.

* The 'plain' format is an as-is 1:1 Serde deserializer, where repeated calls deserialize further values in the file. EOF detection requires calling a custom function to read ahead to determine if EOF has been reached.
* The 'seq' format implements `deserialize_any` via `visitor.visit_seq`, forwards all other methods to that, and has a `SeqAccess` which continues returning values until EOF is detected (which is considered the end of the sequence). It is intended for the List document format.
* The 'map' format implements `deserialize_any` via `visitor.visit_map`, forwards all other methods to that, and has a `MapAccess` which is implemented more or less the same as the 'seq' format. It is intended for the Map document format.

### 'Any'

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
* Unsupported things: `bytes byte_buf`

### Enum

If the next token is a symbol, that symbol is treated as a unit variant.

If it is a list start, then the syntax `(variant ...)` is expected.

If neither a symbol or a list start is found, the value is parsed as per `any`.

### Option

If the next token is `#nil`, then `None` is assumed. Otherwise, `Some` is assumed, and the token is held for parsing the value within.

### Unit

A list start is checked for. If it is found, a list end is expected, and that's the unit.

If a list start is not found, then the value is parsed as per `any`.

`unit_struct` is treated equivalently to `unit`.

It is important that `#nil` and `()` both be valid ways of writing `unit`; the former works in `any` contexts and the latter is valid in Options.

### Map

A list start is checked for. If it is found, a map is visited as per the document layout description above.

If a list start is not found, then the value is parsed as per `any`.

`struct` is treated equivalently to `map`.

### Newtype Struct

`newtype_struct` is simply visited without anything actually parsed by the handler.
