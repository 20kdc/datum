/*
 * datum-rs - Quick to implement S-expression format
 * Written starting in 2024 by contributors (see CREDITS.txt at repository's root)
 * To the extent possible under law, the author(s) have dedicated all copyright and related and neighboring rights to this software to the public domain worldwide. This software is distributed without any warranty.
 * A copy of the Unlicense should have been supplied as COPYING.txt in this repository. Alternatively, you can find it at <https://unlicense.org/>.
 */

use core::{convert::TryFrom, fmt::Write, ops::Deref};

use crate::{datum_error, DatumError, DatumErrorKind, DatumResult, DatumToken};

/// Atomic Datum AST value.
/// This enum also contains the functions that convert between tokens and atoms.
/// You can think of it as the bridge between Datum's tokenization model and value model.
#[derive(Clone, Copy, PartialEq, PartialOrd, Debug)]
pub enum DatumAtom<B: Deref<Target = str>> {
    String(B),
    ID(B),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Nil
}

impl<B: Deref<Target = str>> Default for DatumAtom<B> {
    #[inline]
    fn default() -> Self {
        Self::Nil
    }
}

impl<B: Default + Deref<Target = str>> TryFrom<DatumToken<B>> for DatumAtom<B> {
    type Error = DatumError;

    /// Tries to convert from a DatumToken.
    /// Due to the strings involved, this has to be done via ownership transfer.
    fn try_from(token: DatumToken<B>) -> DatumResult<DatumAtom<B>> {
        match token {
            DatumToken::String(b) => Ok(DatumAtom::String(b)),
            DatumToken::ID(b) => Ok(DatumAtom::ID(b)),
            DatumToken::SpecialID(b) => {
                if b.eq_ignore_ascii_case("t") {
                    Ok(DatumAtom::Boolean(true))
                } else if b.eq_ignore_ascii_case("f") {
                    Ok(DatumAtom::Boolean(false))
                } else if b.eq_ignore_ascii_case("nil") {
                    Ok(DatumAtom::Nil)
                } else if b.eq("{}#") {
                    Ok(DatumAtom::ID(B::default()))
                } else if b.eq_ignore_ascii_case("i+nan.0") {
                    Ok(DatumAtom::Float(f64::NAN))
                } else if b.eq_ignore_ascii_case("i+inf.0") {
                    Ok(DatumAtom::Float(f64::INFINITY))
                } else if b.eq_ignore_ascii_case("i-inf.0") {
                    Ok(DatumAtom::Float(f64::NEG_INFINITY))
                } else if b.starts_with("x") || b.starts_with("X") {
                    let res = i64::from_str_radix(&b[1..], 16);
                    if let Ok(v) = res {
                        Ok(DatumAtom::Integer(v))
                    } else {
                        Err(datum_error!(BadData, "invalid hex integer"))
                    }
                } else {
                    Err(datum_error!(BadData, "invalid special ID"))
                }
            },
            DatumToken::Integer(v) => Ok(DatumAtom::Integer(v)),
            DatumToken::Float(v) => Ok(DatumAtom::Float(v)),
            _ => Err(datum_error!(BadData, "token not atomizable"))
        }
    }
}

impl<B: Deref<Target = str>> DatumAtom<B> {
    /// Writes a value from the atom.
    pub fn write(&self, f: &mut dyn Write) -> core::fmt::Result {
        match &self {
            DatumAtom::String(v) => DatumToken::String(v.deref()).write(f),
            DatumAtom::ID(v) => DatumToken::ID(v.deref()).write(f),
            DatumAtom::Integer(v) => {
                let v: DatumToken<&'static str> = DatumToken::Integer(*v);
                v.write(f)
            },
            DatumAtom::Float(v) => {
                let v: DatumToken<&'static str> = DatumToken::Float(*v);
                v.write(f)
            },
            DatumAtom::Boolean(true) => f.write_str("#t"),
            DatumAtom::Boolean(false) => f.write_str("#f"),
            DatumAtom::Nil => f.write_str("#nil")
        }
    }
}
