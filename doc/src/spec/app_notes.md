# Application Notes

## Expected Document Layouts

Applications using Datum are generally expected to stick to one of five "document layouts".

These layouts may also be embedded in lists.

### List

The document is a list (for example, a list of allowed users, or directory patterns).

No start/end list tokens are used for this outer list, as it is not necessary.

```
; List of files and directories to exclude from export
".git"
".classpath"
```

The list may contain more complex values:

```
; Each excluded entity must have an associated reason
(".git" "metadata, history")
(".classpath" "ide")
```

The list may be a sequence or stream of commands -- see Typical Representations.

### Map

The document is a stream of key/value pairs. The pairs are not explicitly marked with `()`, though if the values are complex then they should be lists (or maps-as-lists in the same style, or maps-as-lists with an identifying symbol).

This model can be very useful for configuration files.

```
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

```
ignore-list
".git"
".classpath"
```

In LISPs, this is only really seen as a value, i.e. `(+ 2 2)`, but the idea is the same.

## Serde Mapping

The Serde mapping defines how Datum interacts with the [serde](https://serde.rs/) Rust crate.

In practice, I expect this to allow quick and immediate use of Datum for configuration files.

**TODO! The Serde mapping is still being defined, and the deserializer has not yet been pulled into the library.**

### Root Deserializers

There are three root deserializers. One is the 'standard' deserializer, and the other two affect the mapping significantly.

* The 'plain' format is an as-is 1:1 Serde deserializer, where repeated calls deserialize further values in the file. EOF detection requires calling a custom function to read ahead to determine if EOF has been reached.
* The 'seq' format implements `deserialize_any` via `visitor.visit_seq`, forwards all other methods to that, and has a `SeqAccess` which continues returning values until EOF is detected (which is considered the end of the sequence). It is intended for the List document format.
* The 'map' format implements `deserialize_any` via `visitor.visit_map`, forwards all other methods to that, and has a `MapAccess` which is implemented more or less the same as the 'seq' format. It is intended for the Map document format.
