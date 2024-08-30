# The Specification

Datum is a data exchange format meant for quick implementation in various languages.
It's designed to be written primarily by humans.

It's described as a series of layers, but some can be merged depending on the implementation's needs.

Certain capitalized words are to be interpreted by their meanings as defined by [RFC 2119](https://www.rfc-editor.org/rfc/rfc2119) and [RFC 8174](https://www.rfc-editor.org/rfc/rfc8174).

If the document is invalid for any reason, implementations can reject it or produce incorrect output (i.e. truncation).

## File Encoding

Datum-formatted data is a stream of 'characters'. Anything ASCII-based is permitted, regardless of character width; non-ASCII codepoints don't play a role in this specification (they are simply content).

The document MUST be valid text for the format it's in.

Characters 0 through 8 inclusive, 11, 12, 14 through 31 inclusive, and 127, are *forbidden;* their presence in the input stream is invalid (can be escaped, except 0).

Character 13 (CR) is discarded, as if it wasn't there (can be escaped).

## Data Model

The following kinds of values exist:

* Symbols and strings are lists of characters.
* Integers are 64-bit signed two's complement integers.
* Doubles are 64-bit floating-point numbers.
* Booleans are true (`#t`) or false (`#f`) values.
* Lists are arrays or linked lists of other values.
* Null (`#nil`) is a null value.

Implementations can have more specific limits on numbers.
Implementations are allowed to reject any file according to resource limits, including but not limited to exceeding the implementation-defined size of internal buffers.

## Encoding

The encoding layer converts characters to a different stream of *class-tagged characters.*

### Escape Sequences

The backslash, 92 `\`, begins an escape sequence, which always produces *content-class* characters.

The backslash must be followed by any valid character, except for newline (10), a specific error.

In addition, these specific characters have special meanings:

* 117 `x`: Followed by >0 hexadecimal digits, terminated by a semicolon 59 `;`. Indicates a Unicode codepoint. MUST be properly terminated, or the document is invalid. MUST NOT escape invalid codepoints, surrogate pairs, or U+00.
* 110 `n`: Newline/10.
* 114 `r`: CR/13.
* 116 `t`: Tab/9.

This provides Datum's escaping logic.

### Character Classes

There are a number of character classes defined here, used in tokenization.

* 10 is *newline-class*.
* 9 (tab) and 32 (space) are *whitespace-class*.
* All *forbidden* characters, along with 13 (CR) and 92 `\`, are *unclassified.*
* 59 `;` is *line-comment-class*.
* 34 `"` is *string-class*.
* 40 `(` is *list-start-class*.
* 41 `)` is *list-end-class*.
* 41 `#` is *special-identifier-class*.
* 45 `-` is *sign-class*.
* 48 `0` through 57 `9` inclusive is *digit-class*.
* All other characters, *including all characters (UTF-8, UTF-16, Unicode, or otherwise) above 127,* are *content-class*.

There are also the following class groups:

* *sign-class* and *digit-class* are the *numeric-start-group*.
* *content-class*, *numeric-start-group*, and *special-identifier-class* are the *potential-identifier-group*.
* *whitespace-class* and *newline-class* are the *non-printing-group*.

## Tokenization

Tokenization, like decoding, is a state machine. However, it's solely defined by character classes.
The expected transformation is from (class, character) pairs to tokens that may contain characters (without classes).

Before reading a token, leading whitespace is consumed in a loop, consisting of:

1. Any *non-printing-group* character.
2. A *line-comment-class* character followed by an arbitrary sequence of characters ending with a *newline-class* character.

Next, multiple kinds of token are possible:

* *Symbol tokens,* a *content-class* character followed by an arbitrary number of *potential-identifier-group* characters, *or* a token that solely consists of a single character of *sign-class*. Examples: `-`, `hello`, `symbol->string`.
* *Numeric tokens,* a *numeric-start-group* character followed by an arbitrary number of *potential-identifier-group* characters, *unless* the token would solely consist of a single *sign-class* character (see *Symbol tokens*). Examples: `12.3`, `-8`.
* *Special Identifier tokens,* a *special-identifier-class* character followed by an arbitrary number of *potential-identifier-group* characters. Example: `#t`.
* *String tokens,* *string-class*-bracketed sequences of any other characters. The only restrictions are that *unclassified* characters or *string-class* characters must be appropriately escaped.
* *list-start-class/list-end-class* characters become *List Start tokens/List End tokens.*

## Grammar & Conversion

* A file is a stream of values.
* Numeric and Special Identifier tokens have special handling after they have been divided into tokens.
* Lists start and end with the appropriately matched list start/end tokens, containing the values between them.

### Special Identifiers

_All of these are 'ASCII case-insensitive'._

* `#{}#`: The empty symbol.
* `#t`: Boolean `true`.
* `#f`: Boolean `false`.
* `#nil`: `null`, etc.
* `#i+inf.0`: Positive infinity float.
* `#i-inf.0`: Negative infinity float.
* `#i+nan.0`: NaN of unspecified kind float.
* `#x` followed by any non-zero number of hex digits of any case: Hexadecimal integer.

### Numbers

Numbers in a document, where not written using the special identifiers above, MUST be of one of the following formats:

* The standard integer format:
	* This is any contiguous sequence of the 10 ASCII decimal digits, which may or may not be preceded by 45 `-`.
		* If the result exceeds the integer limits of the implementation, the resulting value is undefined.

* The standard floating-point format:
	* This is the standard integer format, followed immediately by 46 `.` and then another contiguous sequence of the 10 ASCII decimal digits, such as `0.0` (this does not cover, say, `0.` or `.0`).

* The standard floating-point scientific notation format:
	* This is the standard integer or floating-point format, followed immediately by 101 `e` or 69 `E`, followed by the standard integer format *again.*
	* *Sadly, most programming language standard libraries use this format, so removal is more trouble than it's worth.*

These three formats are the mutual ground between the default integer and floating-point parsing and printing functions of most programming languages.

However, be sure that your implementation doesn't print 'abnormal' forms outside of this. This matters mostly for floating-point values, but can usually be averted by checking for NaNs and infinities and substituting them with special IDs.
