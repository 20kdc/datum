/*
 * datum-rs - Quick to implement S-expression format
 * Written starting in 2024 by contributors (see CREDITS.txt at repository's root)
 * To the extent possible under law, the author(s) have dedicated all copyright and related and neighboring rights to this software to the public domain worldwide. This software is distributed without any warranty.
 * A copy of the Unlicense should have been supplied as COPYING.txt in this repository. Alternatively, you can find it at <https://unlicense.org/>.
 */

//! TODO Rewrite crate outer desc.

// Meta

// should be pretty obvious
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(feature = "unsafe"), forbid(unsafe_code))]
//#![forbid(unsafe_code)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[forbid(unsafe_code)]
mod core_types;
pub use core_types::*;

// Encoding

#[forbid(unsafe_code)]
mod char_classes;
pub use char_classes::*;

#[forbid(unsafe_code)]
mod decoder;
pub use decoder::*;

#[forbid(unsafe_code)]
mod byte_decoder;
pub use byte_decoder::*;

// Tokenization

#[forbid(unsafe_code)]
mod token_core;
pub use token_core::*;

#[forbid(unsafe_code)]
mod token;
pub use token::*;

// Values

#[forbid(unsafe_code)]
mod atom;
pub use atom::*;

// Writing

#[forbid(unsafe_code)]
mod writer;
pub use writer::*;

// AST (alloc-only)

#[forbid(unsafe_code)]
#[cfg(feature = "alloc")]
mod ast;
#[cfg(feature = "alloc")]
pub use ast::*;

#[forbid(unsafe_code)]
#[cfg(feature = "alloc")]
mod pipelines;
#[cfg(feature = "alloc")]
pub use pipelines::*;

// Big test battery

#[forbid(unsafe_code)]
#[cfg(feature = "alloc")]
#[cfg(test)]
mod tests;
