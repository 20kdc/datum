# `datum`: Rust implementation of Datum

Datum is a terse, human-writable data format meant for quick implementation in various languages.

It was originally developed for use in some of my Java programs for the purpose of fulfilling the role of 'terse data language,' with some key distinctions:

* As free-form a syntax as reasonably possible.
* Avoids the problems that YAML has.
* Concise implementation.

It's intended to be reasonably readable by R6RS readers, but not a strict subset. (However, the format has been used in a Java project to implement a "Javaified" Scheme dialect.)

The `datum` crate is a library for reading and writing Datum values in Rust.

In order to allow use in diverse environments, it attempts to follow some key rules:

* `#![no_std]`
* `#![forbid(unsafe_code)]`
* completely public-domain
* no mandatory external dependencies except `libcore`, but without trying to reinvent the wheel _too_ hard
	* optional dependencies are used if it's with good reason, i.e. `serde`

For further information, please see <https://github.com/20kdc/datum>.

## Features

* `std`: Presently mainly for `std::error::Error` support, but may provide additional utilities in future. Mainly, implies `alloc`.
* `alloc`: `DatumStringTokenizer`, `DatumValue` (AST), `DatumParser` (parses tokens into an AST)
* `detailed_errors`: Default feature that includes messages for `DatumError`s. If missing, these will be empty.
* `serde`: Serde support.

Experimental code is gated behind the `_experimental` feature and may be gated behind additional relevant feature flags.

## MSRV/Version Policy

Semantic versioning is in use. However, if at all possible, the major version will never be incremented. If alternate APIs must be created to avoid breaking compatibility, then alternate APIs will be created.

_In the unlikely event that Serde major-versions, the policy will be 'wait-and-see'._

The MSRV is `1.54.0`.

If it comes down to breaking API compatibility or breaking MSRV compatibility, then the MSRV will be updated without a major version bump.

However, _this really shouldn't happen._ The crate should not significantly grow in scope, and the decoupled structure should stop issues like lifetimes getting entangled.
