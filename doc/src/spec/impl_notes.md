# Implementor's Notes

This is an addendum to the Reader's Specification, and covers several key points, particularly around writing.

With that said, it is not a hard requirement, and the specification supersedes it if they conflict.

## Character Classes

Character classes are meant to be implemented using something enum-like.

They can also not be implemented at all, in favour of an 'indirect flag,' but a problem with this appears if/when you wish to write a writer.

The usual API would be along the lines of:

* `identify`: For a character, return a class.
* `isPotentialIdentifier`: If the class is of the *potential-identifier-group*.
* `isNumericStart`: If the class is of the *numeric-start-group*.

Logic can then be written such as `if !isPotentialIdentifier(identify(c)) then escapeChar(c) else writeChar(c)`, along with of course using classes in the tokenizer.

## Control characters

Control characters are formally refused by the specification specifically to allow implementations a lot of leeway in handling them.

This becomes particularly relevant when looking at, say, C versus 'clean' Lua versus Lua using string patterns (where there can be issues with null).

Implementations are expected to take a 'reasonable effort' approach for the language they are written in: Languages that support embedded null characters can (but maybe shouldn't) treat them as any other character, languages that don't support them at all are expected to react poorly (truncate the file, token, etc. as appropriate).

## When To Escape

The rules I've decided should be canonical (i.e. used by the roundtrip tests) are as follows:

* A character is escaped if it would be misinterpreted (backslash or sufficiently 'wrong' class to change tokenization result), is forbidden, or is an ASCII control character (so basically CR/LF/TAB).
* Characters with designated escapes always use them (`\r`, `\n`, `\t`).
* ASCII control characters with no designated escape use 2-digit hex escapes, i.e. `\x7f;`, `\x01;`.
* Any other characters are escaped solely using backslash.

Some notable aspects of the specification that were not explicitly spelled out for brevity in this regard:

* _UTF-16 surrogate pairs are always content-class and are thus never escaped._
* Newline-class characters can be written in the middle of strings. Writing libraries should not machine-write these in the middle of strings by default.
	* It is okay to have code write strings this way if you've got a reason, but this is mainly meant to be an 'end-user formatting choice'.
	* Datum does not have any sort of automatic mid-string indentation removal. If you use strings this way, you're acknowledging it's going to look a bit awkward, and you might have to do some special formatting around it just to make it at all sensible.
