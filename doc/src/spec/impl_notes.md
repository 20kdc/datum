# Implementor's Notes

This is an addendum to the Reader's Specification, and covers several key points, particularly around writing.

## Null Characters

Null characters are formally refused by the specification specifically to allow implementations a lot of leeway in handling them. Implementations are expected to take a 'reasonable effort' approach for the language they are written in; languages that support embedded null characters should handle them as if they were supported by the specification, languages that don't are expected to truncate the file, token, etc. as appropriate.

## When To Escape

Writers should use escapes for any character 0-31 (inclusive) or 127, except when intentionally part of whitespace.

There are various specification requirements that require this for various characters in particular situations.

It is typically useful to have a dedicated function which always writes a character using an escape (be sure to catch `r` `n` `t` `x` for robustness; this typically fits neatly as additional or'd clauses on the condition you'll use to determine if a hex escape is necessary).

## Hex Escapes

An implementation MUST NOT output surrogate pairs from a UTF-16 text as pairs of individual hex escapes. Prefixing a surrogate pair in a UTF-16 output stream with a backslash is okay but usually pointless. Hex-escaping the entire codepoint is perfectly acceptable.

The roundtrip tests assume the writer always uses two-digit hex escapes, where hex escapes are necessary.
