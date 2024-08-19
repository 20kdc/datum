# Reader's Specification

Datum is a data exchange format meant for quick implementation in various languages. It's designed to be written primarily by humans.

It is described as a series of layers, but some layers can be merged depending on the needs of the implementation.

Certain capitalized words are to be interpreted by their meanings as defined by [RFC 2119](https://www.rfc-editor.org/rfc/rfc2119) and [RFC 8174](https://www.rfc-editor.org/rfc/rfc8174).

## File Encoding

Datum-formatted data is a stream of 'characters'. These can be *UTF-8 bytes*, *UTF-32 codepoints*, or *UTF-16 code units.* Non-standard character sets may be used with the appropriate translation of the specification, though there are obviously hazards.

The document MUST be valid text for whatever format it's in. The document MUST NOT have hex escapes that are out of range for UTF-8 (surrogate pairs are out of range) or UTF-16. The document MUST NOT, either via escapes or otherwise, contain null characters.

If the above doesn't apply, the input is formally considered invalid, and implementations can reject these or produce incorrect output (i.e. truncation).

## Data Model

The following kinds of values exist:

* Symbols and strings are lists of characters.
* Integers are 64-bit signed two's complement integers.
* Doubles are 64-bit floating-point numbers. (Some implementations may not differentiate between integers and doubles, so automatic coercion is advised.)
* Booleans are true (`#t`) or false (`#f`) values.
* Lists contain other values.
* Null (`#nil`) is a null value.

Implementations can have more specific limits on numbers, appropriate to their environment of use. Implementations are allowed to reject any file according to resource limits.

## Encoding

The encoding layer converts a stream of characters to a potentially different stream of characters, tagged with *character classes*.

Character values 0 through 8 inclusive, 11, 12, 14 through 31 inclusive, and 127, are *forbidden,* and MUST NOT appear in the input character stream.
They may appear in hex escapes, but for 0 (null) specifically, the results of this are undefined.

13 (CR) is always immediately discarded.

### Escape Sequences

The backslash, 92 `\`, begins an escape sequence, which always produces *content-class* characters.

The backslash may be followed by any valid character, in which case that is the result.
However, these specific characters have special meanings:

* 117 `x`: Followed by a non-zero amount of hexadecimal digits, terminated by a semicolon 59 `;`. Indicates a Unicode codepoint. MUST be properly terminated, or the document is invalid.
* 110 `n`: Newline/10.
* 114 `r`: CR/13.
* 116 `t`: Tab/9.

This provides the escaping logic for the rest of Datum.

### Character Classes

There are a number of character classes defined here, used in tokenization.

* 10 is *newline-class*.
* 9 (tab) and 32 (space) are *whitespace-class*.
* All *forbidden* characters, along with 13 (CR) and 92 `\`, are *meta-class.* These should never reach tokenization; the identification is useful in the decoder and writer.
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
* *list-start-class* and *list-end-class* are the *alone-group*.

## Tokenization

Tokenization, like decoding, is a state machine. However, tokenization is solely defined by character classes.
The expected transformation is from (class, character) pairs to tokens that may contain characters (without classes).

Before reading a token, leading whitespace is consumed in a loop, consisting of:

1. Any *non-printing-group* character.
2. A *line-comment-class* character followed by an arbitrary sequence of characters ending with a *newline-class* character.

Next, multiple kinds of token are possible:

* *Symbol tokens,* a *content-class* character followed by an arbitrary number of *potential-identifier-group* characters, *or* a token that solely consists of a single character of *sign-class*. Examples: `-`, `hello`, `symbol->string`.
	* The `-` token is a special case of Numeric token parsing and is theoretically handled after parsing of a Numeric token completes.
* *Numeric tokens,* a *numeric-start-group* character followed by an arbitrary number of *potential-identifier-group* characters, *unless* the token would solely consist of a single *sign-class* character (see *Symbol tokens*). Examples: `12.3`, `-8`.
* *Special Identifier tokens,* a *special-identifier-class* character followed by an arbitrary number of *potential-identifier-group* characters. Example: `#t`.
* *String tokens,* *string-class*-bracketed sequences of any other characters. The only restrictions are that forbidden characters, *meta-class* characters, or *string-class* characters must be appropriately escaped.
* Characters of the *alone-group* turn into specific token types for each of the group's classes:
	* *list-start-class* characters become *List Start tokens.*
	* *list-end-class* characters become *List End tokens.*

## Tokens To Values

Numeric tokens and Special Identifier tokens have special handling after they have been divided into tokens.

This is because attempting to describe the resulting state machine inline with tokenization is very clearly not worth it.

### Special Identifiers

_All of these are 'ASCII case-insensitive'._

* `#{}#`: This is actually converted into the empty symbol. This mainly exists to remove some of the error cases from writers.
* `#t`: These express the boolean `true` value.
* `#f`: These express the boolean `false` value.
* `#nil`: This represents `null` or so forth. This may or may not be an alias for `()` depending on context.
* `#i+inf.0`: Positive infinity float.
* `#i-inf.0`: Negative infinity float.
* `#i+nan.0`: NaN of unspecified kind float.
* `#x` followed by any non-zero number of hex digits of any case: Hexadecimal integer.

### Numbers

Numbers in a document MUST be of one of the following formats:

* The standard integer format, which *must* be supported:
	* This is any contiguous sequence of the 10 ASCII decimal digits, which may or may not be preceded by 45 `-` or 43 `+` (*this should rarely come up due to how parsing has been defined but is important*).
		* If the result exceeds the integer limits of the implementation, the resulting value is undefined.
	* If the source data model makes no distinction whatsoever between floating point and integer values (i.e. it doesn't have integers, period), this format *should* be used whenever it would not lose precision, unless specified otherwise.

* The standard floating-point format. If floating point values are supported by the implementation, this format *must* be supported:
	* This is the standard integer format, followed immediately by 46 `.` and then another contiguous sequence of the 10 ASCII decimal digits, such as `0.0` (this does not cover, say, `0.` or `.0`).

* The standard floating-point scientific notation format. This format *should* be supported:
	* This is the standard integer or floating-point format, followed immediately by 101 `e` or 69 `E`, followed by the standard integer format *again.*
	* *Unfortunately, most programming language standard libraries will use this format under some set of conditions, and they make it rather difficult to override.*
		* It is possible to write a function to fix these, but doing so also goes somewhat against the minimal-implementation ideals of Datum.
			* In addition, `1e+308` is representable as a 64-bit floating-point number. The implication is that the results may be... amusing.

These three formats are the mutual ground between the default integer and floating-point parsing and printing functions of most programming languages.

However, do be sure that your language of choice does not print 'abnormal' forms outside of this. This is a particular danger for floating-point values, but can mainly be averted by checking for NaNs and infinities, which must be substituted with the appropriate constants.

## Grammar

The grammar of Datum is very simple:

* A file is a stream of values (or alternatively, it can be seen as one big implicit list).
* Tokens that can be converted directly to values become those values.
* Lists start with the start-list token `(` and end with the end-list token `)`.
