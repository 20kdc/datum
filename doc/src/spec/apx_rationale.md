# Datum Specification

## Why not a pure R6RS subset?

* Minimalism. While attempting to be a specification-compliant Scheme reader for various languages would certainly be novel, Scheme has a lot of functionality that screams 'bloat' -- especially when it is being used as a data interchange format rather than as part of Scheme. Consider, for instance, complex numbers. Blocking every single possible situation where a Datum reader could possibly misinterpret non-Scheme-compliant input would be very complicated and heavily worsen the spec.
* Datum attempts to ensure that write/read is a reliable operation, even for very non-standard values that "shouldn't really exist", but are possible because the datamodel has to allow for extensibility in the event someone tries to use 128-bit numbers or something. For instance, `#i` is an empty numeric token.
    * `#i` became an empty numeric token during the Rust implementation, due to a problem: It was possible to create unwritable numeric tokens. The sensible options were to return an error (easily unexpected, and very potentially `.unwrap()`'d under the assumption that writing to a string is infallible) or to change the spec. I changed the spec.

## If minimalism is the goal, why are there hex escapes and hex integers? What are special identifiers for?

* Hex character escapes are, more than anything else, a concession to unprintable characters and general Unicode jank. Think zero-width-joiners.
* Hex integers are a concession to the continued ubiquity of bit manipulation.
* Special identifiers exist as a Scheme-compatible-ish place to put these things.

## Why aim for Scheme compatibility at all, in any sense?

Aiming for some semblance of Scheme compatibility comes with free stuff, like decent highlighting rules across a wide variety of editors.

Also, while it can't be a pure Scheme subset for various reasons (up to and including empty symbols not being standardized!), Datum files which aren't too obnoxious should be readable by compliant Scheme interpreters. (The main problem I have encountered so far with this theory is that Guile does not agree with R6RS or R7RS on the matter of inline hex escapes.)

Compliant Scheme interpreters which are also not too obnoxious should also produce valid Datum, but I would recommend not abusing this in any security-critical situation, as the misinterpretation of an escape in that scenario could lead to a lot of very bad things.

The history also helps; before I decided to try and make it into a more generally portable data format, I was using Datum as part of a Scheme dialect for embedding into a Java application.

Enough is there that implementing minimal Schemes based on Datum should be viable, though the lack of prefixes for things like quasi-quoting may hurt.

## Why let the unit of decoding be ambiguous between Unicode codepoints and bytes? Why is null handling ambiguous?

Different languages have different "convenient units".

* Lua, C and Zig use null-terminated byte pointers.
* C++ either does what C does or uses byte pointers which are not _treated_ as though they're null terminated, but they keep a null terminator around anyway for the C code that they'll inevitably run into.
* Java and C# use UTF-16 and treat nulls as just data.
* Rust uses byte slices, without a null terminator, that are asserted to be valid UTF-8 (and will panic if a slice is attempted that would create invalid UTF-8, even transiently). Nulls are treated as just data.

Specifying any specific treatment would put cost on languages which don't follow that exact treatment.

In addition, there is a fundamental, insurmountable problem with using any data gleaned from Unicode tables inside any form of machine-readable language: _Unicode tables change._

* For example, `🨂` is not a valid identifier in Java, the normal C compilers, and Python. There is no particular justifiable reason for this except that Unicode doesn't consider it a letter. If Unicode were to consider it a letter in future, the result could be effectively a version break in Java and Python (the normal C compilers do their own thing here).
* The behaviour of `gcc (Ubuntu 11.3.0-1ubuntu1~22.04) 11.3.0` appears to indicate GCC Unicode identifier compatibility operates by exclusion, i.e. `U+3FF80` is a valid identifier character. Java and Python, meanwhile, consider `U+3FF80` (Unassigned as of Unicode 15.0) invalid, but `U+10400` (Deseret: 𐐀) valid. A hypothetical future Unicode version could therefore enable valid Java and Python identifier characters that past versions refuse to accept for reasons that are, frankly, completely arbitrary. GCC's behaviour, on the other hand, could lead to code becoming invalid on a similar basis. This would be arguably worse if not for that people do not just arbitrarily use unassigned codepoints.
* GCC, Python and Java do consider the *private use areas* as invalid, for some unknown reason. (This rather defeats the purpose of the private use area as it is presently used.)
* ICU soversioning is a complete disaster. Case in point... <https://github.com/dotnet/runtime/blob/217525ae6f6a117a0780620ed4fb1b94e03fd4d6/src/native/libs/System.Globalization.Native/pal_icushim.c#L201> _This is perfectly reasonable code for an unreasonable situation._

## Why is the decoder separate from the tokenizer?

The decoder is separate from the tokenizer for three key reasons:

1. It simplifies the specification over having three separate sets of 'escaping while...' states. In simpler 'read(1)' implementations, the decoder can be a single function that lives close to the tokenizer, also responsible for returning the tokenizer's 'unput'. It can either provide a character class with a character (identification in decoder; more upfront code but easier to read tokenizer), or it can provide a direct/indirect flag with a character (identification in tokenizer; less upfront code -- only really needs a function to check if a character is a potential identifier -- but tokenizer is much more awkward). As it already manages the 'unput byte,' this buffer can be extended to also include the output of Unicode escapes.
2. It is important to be able to write any value to prevent 'surprise errors', where data, having been transformed, is re-emitted; but is now not writable, and thus an error occurs.
3. Implementing the decoder as a clear, delineated unit allows for tricks like zero-copy parsing. In 'push' models, by using them individually and monitoring their output, the decoder and tokenizer can provide the information to slice the input text for numeric, string, and ID tokens. The resulting slices can then be wrapped in a datastructure to re-decode them on-demand.
