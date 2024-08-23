# Datum: Terse, human-writable data format

**Under construction!**

_It feels to me that 'Release early, release often' would be a fatal blow in certain languages (Rust), so expect work on this to continue in the coming weeks._

Datum is a terse, human-writable data format meant for quick implementation in various languages.

It has a specification, available at [./doc/src/spec](./doc/src/spec).

It was originally developed for use in some of my Java programs for the purpose of fulfilling the role of 'terse data language,' with some key distinctions:

* As free-form a syntax as reasonably possible.
* Avoids the problems that YAML has.
* Concise implementation.

It's intended to be reasonably readable by R6RS readers, but not a strict subset. (However, it has been used in a Java project to implement a "Javaified" Scheme dialect.)

Implementations exist for:

* Java (*not yet stabilized*)
* Rust (*not yet stabilized*)

## TODO

* Use `rustyline` correctly in example code
* Figure out datum-rs source positioning
* Figure out how to make doctests that rely on alloc while not being for alloc functions?
* Go over datum-rs with a fine-toothed comb
* Do the libraries need to go on a diet?
    * Java: Way too many kinds of Visitor specific to all the very niche cases that came up in use in GaBIEn
    * Rust: So I'm convinced not using async is a mistake (because of the wide variety of state machines that are better with control flow) and I'm also convinced using async would be a mistake (because it doesn't do *that* much when you account for necessary overhead). FIGURE THIS OUT.
* Java impl needs Javadocs
* Rename Java classes to avoid confusion on meaning of 'decode'
* Shore up all the documentation
* Simultaneous release of v1.0.0 for Java and Rust

## License

```
This is free and unencumbered software released into the public domain.

Anyone is free to copy, modify, publish, use, compile, sell, or
distribute this software, either in source code form or as a compiled
binary, for any purpose, commercial or non-commercial, and by any
means.

In jurisdictions that recognize copyright laws, the author or authors
of this software dedicate any and all copyright interest in the
software to the public domain. We make this dedication for the benefit
of the public at large and to the detriment of our heirs and
successors. We intend this dedication to be an overt act of
relinquishment in perpetuity of all present and future rights to this
software under copyright law.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
IN NO EVENT SHALL THE AUTHORS BE LIABLE FOR ANY CLAIM, DAMAGES OR
OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE,
ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR
OTHER DEALINGS IN THE SOFTWARE.

For more information, please refer to <http://unlicense.org>
```
