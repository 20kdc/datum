# Rust Implementation

Datum's Rust implementation, `datum_rs`, is designed to work in a variety of environments.

In particular:

* `no_std` but `alloc` is easy to deal with.
* `no_std` and no `alloc` is a lot harder to deal with.
	* If you do this, you will need to implement a 'limited-size string' datatype, or fake one with a careful state machine containing all strings your program is expected to handle, along with numbers, and try to avoid the various edge cases and issues you can encounter doing that.
		* If you have a type implementing `Write + Deref<Target = str> + Default` (`String` is of course such a type), you're set.
	* You also lose access to the built-in AST support, though this isn't too bad (build a visitor pattern using the `Iterator::via_datum_pipe` API).

While most of the library is documented with documentation comments, some of the core types are worth going over, along with examples.
