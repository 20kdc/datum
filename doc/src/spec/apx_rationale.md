# Appendix: Rationale

## What about the security implications of all of this leeway?

Datum is designed to be written primarily by humans and used in scenarioes where different interpretations are not a security-relevant factor.

However, if you believe your application of Datum is at risk due to different interpretation by different implementations, require that the data be pre-processed by one implementation in a consistent fashion according to the Writer's Specification gudelines. All receiving implementations must then decode and re-serialize the data, again according to the Writer's Specification. Should the output not match the original input, there is something wrong and the implementations do not agree. Reject the data.

## Why not a pure R6RS subset?

Minimalism.

While attempting to be a specification-compliant Scheme reader for various languages would certainly be novel, Scheme has a lot of functionality that screams 'bloat' -- especially when it is being used as a data interchange format rather than as part of Scheme.

Consider, for instance, complex numbers. Blocking every single possible situation where a Datum reader could possibly misinterpret non-Scheme-compliant input would be very complicated and heavily worsen the spec.

However, a lot of restrictions are present that should 'more or less' keep it close enough to a subset for practical use.

## If minimalism is the goal, why are there hex escapes and hex integers? What are special identifiers for?

* Hex character escapes are, more than anything else, a concession to unprintable characters and general Unicode jank. Think zero-width-joiners.
* Hex integers are a concession to the continued ubiquity of bit manipulation.
* Special identifiers exist as a Scheme-compatible-ish place to put these things.

## Why aim for Scheme compatibility at all, in any sense?

Aiming for some semblance of Scheme compatibility comes with free stuff, like decent highlighting rules across a wide variety of editors.

Also, while it can't be a pure Scheme subset for various reasons (up to and including empty symbols not being standardized!), Datum files which aren't too obnoxious should be readable by compliant Scheme interpreters. (The main problem I have encountered so far with this theory is that Guile does not agree with R6RS or R7RS on the matter of inline hex escapes.)

Compliant Scheme interpreters which are also not too obnoxious should also produce valid Datum.

The history also helps; before I decided to try and make it into a more generally portable data format, I was using Datum as part of a Scheme dialect for embedding into a Java application.

Enough is there that implementing minimal Scheme dialects based on Datum should be viable, though there are two caveats:

* The lack of prefixes for things like quasi-quoting may hurt.
* There is no such thing as the `.` notation.

## Why let the unit of decoding be ambiguous between Unicode codepoints and bytes? Why is null handling ambiguous?

Firstly, different languages have different "convenient units".

* Lua uses byte slices of an arbitrary character set. In later versions, it is implied to be expected that this is UTF-8, but it is not guaranteed. Indeed, on Windows, it is likely that filenames in particular use the "ANSI character set" of the system for some versions of Lua.
* C and Zig use null-terminated byte pointers.
* C++ either does what C does or uses byte pointers which are not _treated_ as though they're null terminated, but they keep a null terminator around anyway for the C code that they'll inevitably run into.
	* This does not strictly apply to frameworks such as Qt.
* Java and C# use UTF-16 and treat nulls as just data.
* Rust uses byte slices, without a null terminator, that are asserted to be valid UTF-8 (and will panic if a slice is attempted that would create invalid UTF-8, even transiently). Nulls are treated as just data.

For valid files that do not contain nulls, behaviour is expected to be identical regardless of which of these is chosen; any situation where the difference matters would simply replace a single content-class character with multiple, and there are no situations in the specification where this can have any effect on the outcome.

Specifying any specific treatment would put cost on languages which don't follow that exact treatment.

Secondly, this is part of a choice to ensure that reading Datum never requires UTF-8 or UTF-16 to be decoded, though encoding may be required for hex escapes.

In addition, there is a fundamental, insurmountable problem with using any data gleaned from Unicode tables inside any form of machine-readable language: _Unicode tables change._

* For example, `🨂` is not a valid identifier in Java, the normal C compilers, and Python. There is no particular justifiable reason for this except that Unicode doesn't consider it a letter. If Unicode were to consider it a letter in future, the result could be effectively a version break in Java and Python (the normal C compilers do their own thing here).
* The behaviour of `gcc (Ubuntu 11.3.0-1ubuntu1~22.04) 11.3.0` appears to indicate GCC Unicode identifier compatibility operates by exclusion, i.e. `U+3FF80` is a valid identifier character. Java and Python, meanwhile, consider `U+3FF80` (Unassigned as of Unicode 15.0) invalid, but `U+10400` (Deseret: 𐐀) valid. A hypothetical future Unicode version could therefore enable valid Java and Python identifier characters that past versions refuse to accept for reasons that are, frankly, completely arbitrary. GCC's behaviour, on the other hand, could lead to code becoming invalid on a similar basis. This would be arguably worse if not for that people do not just arbitrarily use unassigned codepoints.
* GCC, Python and Java do consider the *private use areas* as invalid, for some unknown reason. (This rather defeats the purpose of the private use area as it is presently used.)
* ICU soversioning is a complete disaster. Case in point... <https://github.com/dotnet/runtime/blob/217525ae6f6a117a0780620ed4fb1b94e03fd4d6/src/native/libs/System.Globalization.Native/pal_icushim.c#L201> _This is perfectly reasonable code for an unreasonable situation._

## Why is the decoder separate from the tokenizer?

The decoder is separate from the tokenizer for three key reasons:

1. It simplifies the specification over having three separate sets of 'escaping while...' states. In simpler 'read(1)' implementations, the decoder can be a single function that lives close to the tokenizer, also responsible for returning the tokenizer's 'unput'. It can either provide a character class with a character (identification in decoder; more upfront code but easier to read tokenizer), or it can provide a direct/indirect flag with a character (identification in tokenizer; less upfront code -- only really needs a function to check if a character is a potential identifier -- but tokenizer is much more awkward). As it already manages the 'unput byte,' this buffer can be extended to also include the output of Unicode escapes.
2. It is important to be able to write any value to prevent 'surprise errors', where data, having been transformed, is re-emitted; but is now not writable, and thus an error occurs.
3. Implementing the decoder as a clear, delineated unit allows for tricks like zero-copy parsing. In 'push' models, by using them individually and monitoring their output, the decoder and tokenizer can provide the information to slice the input text for numeric, string, and symbol tokens. The resulting slices can then be wrapped in a datastructure to re-decode them on-demand.

## Why does number/special identifier parsing occur after tokenization?

This is mainly due to a sort of 'fight' between the standard library of many languages and zero-copy parsing.

Ultimately, the way things are now, zero-alloc-focused implementations which can pass handling of this stage to the user have a pretty decent chance of being able to parse all integers and special identifiers 'live' without resorting to reimplementing the entire tokenizer or fixed-size buffers (with the implicit limit on numeric token size).

In the Rust implementation, this manifests itself as the `Push`/`Token` tokenizer action stream.
The user can track the 'expected integer value' of whatever is being written via `Push`, and then commit that if the appropriate `Token` action appears.
Errors from the traditional buffer can be deferred until the token is returned.

Float parsing, however, is hard, and mistakes can be very, very subtle, and very, very bad. For this reason, no generalized 'streaming number parser' implementation exists in Datum.
