# What (And Why) Is Datum?

Datum is a data format, similar to JSON or YAML.

It was originally developed for use in some of my Java programs for the purpose of fulfilling the role of 'terse data language,' with some key distinctions.

## Versus JSON

* Comments. Trailing commas are not an error. (I am aware of JSON5, but this does not change the state of JSON parsers as are commonly available.)
* Significantly less "syntactic spam" (commas, colons, etc.) inherited from JavaScript.
	* `[1, 2, 3]` becomes `(1 2 3)`.
* Datum has a _Symbol_ type. LISP-style or Scheme-style languages naturally fit into Datum for use in templating.
* Datum does not have a dictionary type.
* Datum has a distinction between integers and floating-point numbers.

## Versus YAML

* Concise specification, concise implementation.
