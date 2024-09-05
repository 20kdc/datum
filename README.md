# Datum: Terse, human-writable data format

**Under construction!** _See TODO._

Datum is a terse, human-writable data format meant for quick implementation in various languages.

It has a specification, available at [./doc/src/spec](./doc/src/spec).

Implementations exist for:

* Java [./java](./java) (*not yet stabilized*)
* Rust [./rust](./rust)

It was originally developed for use in some of my Java programs for the purpose of fulfilling the role of 'terse data language,' with some key distinctions:

* As free-form a syntax as reasonably possible.
* Avoids the problems that YAML has.
* Concise implementation.

It's intended to be reasonably readable by R6RS readers, but not a strict subset. (However, it has been used in a Java project to implement a "Javaified" Scheme dialect.)

## TODO

* Does the Java version of the library need to go on a diet?
    * Java: Way too many kinds of Visitor specific to all the very niche cases that came up in use in GaBIEn.
* Java impl needs Javadocs
* Rename Java classes to avoid confusion on meaning of 'decode'
* Java version's v1.0.0?

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
