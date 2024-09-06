# History of `cargo yank`'d versions of the crate

## v1.0.1

This just fixed some documentation and cleaned up lib.rs. Unfortunately, the no_std guarantee was lost in that cleanup, and I didn't have a mechanism to catch it.

Ultimately since v1.0.0 exists and no APIs were added, v1.0.1 was yanked, which basically fixes everything.
