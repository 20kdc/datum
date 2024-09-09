/*
 * datum-rs - Quick to implement S-expression format
 * Written starting in 2024 by contributors (see CREDITS.txt at repository's root)
 * To the extent possible under law, the author(s) have dedicated all copyright and related and neighboring rights to this software to the public domain worldwide. This software is distributed without any warranty.
 * A copy of the Unlicense should have been supplied as COPYING.txt in this repository. Alternatively, you can find it at <https://unlicense.org/>.
 */

//! The `datum` crate provides reading/writing utilities for the human-writable data format of the same name.
//! For further information, see <https://github.com/20kdc/datum> and the documentation there.

// Meta

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]

#[cfg(feature = "alloc")]
extern crate alloc;

mod errors;
pub use errors::*;

pub mod unary;

mod queue;
pub use queue::*;

mod pipe;
pub use pipe::*;

mod pipe_vdp;
pub use pipe_vdp::*;

// Encoding

mod char_classes;
pub use char_classes::*;

mod decoder;
pub use decoder::*;

mod byte_decoder;
pub use byte_decoder::*;

// Tokenization

mod token_core;
pub use token_core::*;

mod token;
pub use token::*;

// Values

mod atom;
pub use atom::*;

// Writing

mod writer;
pub use writer::*;

// AST (alloc-only)

#[cfg(feature = "alloc")]
mod ast;
#[cfg(feature = "alloc")]
pub use ast::*;

// Pipelines (partially alloc-only)

mod pipelines;
pub use pipelines::*;

// Big test battery

#[cfg(feature = "alloc")]
#[cfg(test)]
mod tests;

// Integrations

#[cfg(feature = "serde")]
pub mod serde;
