# Implementor's Notes

This is an addendum to the Reader's Specification, and covers several key points, particularly around writing.

## Character Classes

Character classes are meant to be implemented using something enum-like.

They can also not be implemented at all, in favour of an 'indirect flag,' but a problem with this appears if/when you wish to write a writer.

The usual API would be along the lines of:

* `identify`: For a character, return a class.
* `isPotentialIdentifier`: If the class is of the *potential-identifier-group*.
* `isNumericStart`: If the class is of the *numeric-start-group*.

Logic can then be written such as `if !isPotentialIdentifier(identify(c)) then escapeChar(c) else writeChar(c)`, along with of course using classes in the tokenizer.

## Null Characters

Null characters are formally refused by the specification specifically to allow implementations a lot of leeway in handling them. Implementations are expected to take a 'reasonable effort' approach for the language they are written in; languages that support embedded null characters should handle them as if they were supported by the specification, languages that don't are expected to truncate the file, token, etc. as appropriate.

## When To Escape

Writers should use escapes for any character 0-31 (inclusive) or 127, except when intentionally part of whitespace.

There are various specification requirements that require this for various characters in particular situations.

It is typically useful to have a dedicated function which always writes a character using an escape (be sure to catch `r` `n` `t` `x` for robustness; this typically fits neatly as additional or'd clauses on the condition you'll use to determine if a hex escape is necessary).

## Hex Escapes

An implementation MUST NOT output surrogate pairs from a UTF-16 text as pairs of individual hex escapes. Prefixing a surrogate pair in a UTF-16 output stream with a backslash is okay but usually pointless. Hex-escaping the entire codepoint is perfectly acceptable.

The roundtrip tests assume the writer always uses two-digit hex escapes, where hex escapes are necessary.
