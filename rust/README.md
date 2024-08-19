# `datum-rs`

Datum is an S-expression format meant for quick implementation in various languages.

It has a specification, available at <https://github.com/20kdc/gabien-common/blob/master/datum/specification.md>.

It was originally developed for use in some of my Java programs for the purpose of fulfilling the role of 'terse data language,' with some key distinctions:

* As free-form a syntax as reasonably possible.
* Avoids the problems that YAML has.
* Concise implementation.

It's intended to be reasonably readable by R6RS readers, but not a strict subset. (However, it has been used in a Java project to implement a "Javaified" Scheme dialect.)

`datum-rs` is a library for reading and writing Datum values in Rust.

In order to allow use in diverse environments, it attempts to follow some key rules:

* `#![no_std]`
* `#![forbid(unsafe_code)]`
* completely public-domain
* no external dependencies, but without trying to reinvent the wheel _too_ hard

With that said, it doesn't implement `serde` support at present.

## TODO

* Example program (a toy LISP perhaps?)
* Shore up all the documentation
* Figure out how to make doctests that rely on alloc while not being for alloc functions
* If Datum is going to be a serious project I should probably move it out of the `gabien-common` umbrella. But also, that's probably going to mean the Java implementation has to be either left behind or things are going to get a little ugly. Basically, mass reorganization incoming.

## Features

* `std`: Currently a breakage prevention placeholder as it isn't used right now.
* `alloc`: For if alloc is used.
* `detailed_errors`: This adds compile-time location/line strings for all `DatumError`s. If missing, these will be empty.
