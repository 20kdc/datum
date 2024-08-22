/*
 * datum-rs - Quick to implement S-expression format
 * Written starting in 2024 by contributors (see CREDITS.txt at repository's root)
 * To the extent possible under law, the author(s) have dedicated all copyright and related and neighboring rights to this software to the public domain worldwide. This software is distributed without any warranty.
 * A copy of the Unlicense should have been supplied as COPYING.txt in this repository. Alternatively, you can find it at <https://unlicense.org/>.
 */

use core::{convert::TryFrom, fmt::{Display, Write}, ops::Deref};

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

impl<B: Deref<Target = str>> Display for DatumAtom<B> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.write(f)
    }
}

/// Implemented by structs which may contain an Atom to give easy access to said Atom.
pub trait DatumMayContainAtom<B: Deref<Target = str>> {
    /// Retrieves a reference to the possible interior [DatumAtom].
    fn as_atom(&self) -> Option<&DatumAtom<B>>;
    /// Retrieves a mutable reference to the possible interior [DatumAtom].
    fn as_atom_mut(&mut self) -> Option<&mut DatumAtom<B>>;
    /// From the interior [DatumAtom] (if any), retrieves the string (if it is), else [None].
    fn as_str(&self) -> Option<&B> {
        if let Some(DatumAtom::String(res)) = self.as_atom() {
            Some(res)
        } else {
            None
        }
    }
    /// From the interior [DatumAtom] (if any), retrieves the symbol (if it is), else [None].
    fn as_id(&self) -> Option<&B> {
        if let Some(DatumAtom::ID(res)) = self.as_atom() {
            Some(res)
        } else {
            None
        }
    }
    /// From the interior [DatumAtom] (if any), retrieves the int (if it is), else [None].
    fn as_i64(&self) -> Option<i64> {
        if let Some(DatumAtom::Integer(res)) = self.as_atom() {
            Some(*res)
        } else {
            None
        }
    }
    /// From the interior [DatumAtom] (if any), retrieves the float (if it is), else [None].
    fn as_f64(&self) -> Option<f64> {
        if let Some(DatumAtom::Float(res)) = self.as_atom() {
            Some(*res)
        } else {
            None
        }
    }
    /// From the interior [DatumAtom] (if any), retrieves the float (if it is), else [None].
    /// This version will cast integers to floats if necessary.
    fn as_number(&self) -> Option<f64> {
        if let Some(res) = self.as_atom() {
            if let DatumAtom::Float(res) = res {
                Some(*res)
            } else if let DatumAtom::Integer(res) = res {
                Some(*res as f64)
            } else {
                None
            }
        } else {
            None
        }
    }
    /// From the interior [DatumAtom] (if any), retrieves the boolean (if it is), else [None].
    fn as_bool(&self) -> Option<bool> {
        if let Some(DatumAtom::Boolean(res)) = self.as_atom() {
            Some(*res)
        } else {
            None
        }
    }
    /// From the interior [DatumAtom] (if any), returns [Some] if a nil value, else [None].
    fn as_nil(&self) -> Option<()> {
        if let Some(DatumAtom::Nil) = self.as_atom() {
            Some(())
        } else {
            None
        }
    }
}

impl<B: Deref<Target = str>> DatumMayContainAtom<B> for DatumAtom<B> {
    fn as_atom(&self) -> Option<&DatumAtom<B>> {
        Some(self)
    }

    fn as_atom_mut(&mut self) -> Option<&mut DatumAtom<B>> {
        Some(self)
    }
}
