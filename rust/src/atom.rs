/*
 * datum-rs - Quick to implement S-expression format
 * Written starting in 2024 by contributors (see CREDITS.txt at repository's root)
 * To the extent possible under law, the author(s) have dedicated all copyright and related and neighboring rights to this software to the public domain worldwide. This software is distributed without any warranty.
 * A copy of the Unlicense should have been supplied as COPYING.txt in this repository. Alternatively, you can find it at <https://unlicense.org/>.
 */

use core::{
    convert::TryFrom,
    fmt::{Display, Write},
    hash::Hash,
    ops::Deref,
};

use crate::{datum_error, DatumError, DatumResult, DatumToken};

/// Atomic Datum AST value.
/// This enum also contains the functions that convert between tokens and atoms.
/// You can think of it as the bridge between Datum's tokenization model and value model.
/// Accordingly, it does not contain offsets.
/// Implements [Hash] despite potentially containing floats; if this is a problem for your application then don't use the [Hash] implementation.
#[derive(Clone, Copy, PartialEq, PartialOrd, Debug)]
pub enum DatumAtom<B: Deref<Target = str>> {
    String(B),
    Symbol(B),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Nil,
}

impl<B: Deref<Target = str>> Default for DatumAtom<B> {
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
            DatumToken::String(_, b) => Ok(DatumAtom::String(b)),
            DatumToken::Symbol(_, b) => Ok(DatumAtom::Symbol(b)),
            DatumToken::SpecialID(at, b) => {
                if b.eq_ignore_ascii_case("t") {
                    Ok(DatumAtom::Boolean(true))
                } else if b.eq_ignore_ascii_case("f") {
                    Ok(DatumAtom::Boolean(false))
                } else if b.eq_ignore_ascii_case("nil") {
                    Ok(DatumAtom::Nil)
                } else if b.eq("{}#") {
                    Ok(DatumAtom::Symbol(B::default()))
                } else if b.eq_ignore_ascii_case("i+nan.0") {
                    Ok(DatumAtom::Float(f64::NAN))
                } else if b.eq_ignore_ascii_case("i+inf.0") {
                    Ok(DatumAtom::Float(f64::INFINITY))
                } else if b.eq_ignore_ascii_case("i-inf.0") {
                    Ok(DatumAtom::Float(f64::NEG_INFINITY))
                } else if b.starts_with('x') || b.starts_with('X') {
                    let res = i64::from_str_radix(&b[1..], 16);
                    if let Ok(v) = res {
                        Ok(DatumAtom::Integer(v))
                    } else {
                        Err(datum_error!(BadData, at, "invalid hex integer"))
                    }
                } else {
                    Err(datum_error!(BadData, at, "invalid special ID"))
                }
            }
            DatumToken::Integer(_, v) => Ok(DatumAtom::Integer(v)),
            DatumToken::Float(_, v) => Ok(DatumAtom::Float(v)),
            _ => Err(datum_error!(
                BadData,
                token.offset(),
                "token not atomizable"
            )),
        }
    }
}

impl<B: Deref<Target = str>> DatumAtom<B> {
    /// Writes a value from the atom.
    pub fn write(&self, f: &mut dyn Write) -> core::fmt::Result {
        match &self {
            DatumAtom::String(v) => DatumToken::String(0, v.deref()).write(f),
            DatumAtom::Symbol(v) => DatumToken::Symbol(0, v.deref()).write(f),
            DatumAtom::Integer(v) => {
                let v: DatumToken<&'static str> = DatumToken::Integer(0, *v);
                v.write(f)
            }
            DatumAtom::Float(v) => {
                let v: DatumToken<&'static str> = DatumToken::Float(0, *v);
                v.write(f)
            }
            DatumAtom::Boolean(true) => f.write_str("#t"),
            DatumAtom::Boolean(false) => f.write_str("#f"),
            DatumAtom::Nil => f.write_str("#nil"),
        }
    }
}

impl<B: Deref<Target = str>> Display for DatumAtom<B> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.write(f)
    }
}

impl<B: Deref<Target = str>> Hash for DatumAtom<B> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        match self {
            Self::String(s) => {
                state.write_u8(0);
                s.hash(state)
            }
            Self::Symbol(s) => {
                state.write_u8(1);
                s.hash(state)
            }
            Self::Integer(i) => {
                state.write_u8(2);
                i.hash(state)
            }
            Self::Float(f) => {
                state.write_u8(3);
                f.to_bits().hash(state)
            }
            Self::Boolean(b) => {
                state.write_u8(4);
                b.hash(state)
            }
            Self::Nil => state.write_u8(5),
        }
    }
}

macro_rules! as_x_result {
    ($caller:ident, $callee:ident, $type:ty) => {
        #[doc = concat!("Wraps [DatumMayContainAtom::", stringify!($callee), "] to return [Result], using the given error generator.")]
        fn $caller<E, F: FnOnce() -> E>(&self, err: F) -> Result<$type, E> {
            match self.$callee() {
                Some(v) => Ok(v),
                None => Err(err())
            }
        }
    };
}

macro_rules! as_x {
    ($name:ident, $name_result:ident, $variant:ident, $type:ty) => {
        #[doc = concat!("Attempts to retrieve [DatumAtom::", stringify!($variant), "], else [None].")]
        fn $name(&self) -> Option<$type> {
            if let Some(DatumAtom::$variant(res)) = self.as_atom() {
                Some(res)
            } else {
                None
            }
        }
        as_x_result!($name_result, $name, $type);
    };
    ($name:ident, $name_result:ident, $variant:ident, $type:ty, $adjust:tt) => {
        #[doc = concat!("Attempts to retrieve [DatumAtom::", stringify!($variant), "], else [None].")]
        fn $name(&self) -> Option<$type> {
            if let Some(DatumAtom::$variant(res)) = self.as_atom() {
                Some($adjust res)
            } else {
                None
            }
        }
        as_x_result!($name_result, $name, $type);
    };
}

/// Implemented by structs which may contain an Atom to give easy access to said Atom.
pub trait DatumMayContainAtom<B: Deref<Target = str>> {
    /// Retrieves a reference to the possible interior [DatumAtom].
    fn as_atom(&self) -> Option<&DatumAtom<B>>;
    as_x_result!(as_atom_result, as_atom, &DatumAtom<B>);
    as_x!(as_str, as_str_result, String, &B);
    as_x!(as_sym, as_sym_result, Symbol, &B);
    as_x!(as_i64, as_i64_result, Integer, i64, *);
    as_x!(as_f64, as_f64_result, Float, f64, *);
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
    as_x_result!(as_number_result, as_number, f64);
    as_x!(as_bool, as_bool_result, Boolean, bool, *);
    /// From the interior [DatumAtom] (if any), returns [Some] if a nil value, else [None].
    fn as_nil(&self) -> Option<()> {
        if let Some(DatumAtom::Nil) = self.as_atom() {
            Some(())
        } else {
            None
        }
    }
    as_x_result!(as_nil_result, as_nil, ());
}

impl<B: Deref<Target = str>> DatumMayContainAtom<B> for DatumAtom<B> {
    fn as_atom(&self) -> Option<&DatumAtom<B>> {
        Some(self)
    }
}
