/*
 * datum-rs - Quick to implement S-expression format
 * Written starting in 2024 by contributors (see CREDITS.txt at repository's root)
 * To the extent possible under law, the author(s) have dedicated all copyright and related and neighboring rights to this software to the public domain worldwide. This software is distributed without any warranty.
 * A copy of the Unlicense should have been supplied as COPYING.txt in this repository. Alternatively, you can find it at <https://unlicense.org/>.
 */

//! [serde::Serializer] implementation and supporting types.
//!
//! _Added in 1.1.0._

mod serializer;
pub use serializer::*;
mod seqmaproot;
pub use seqmaproot::*;
