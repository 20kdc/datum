/*
 * datum-rs - Quick to implement S-expression format
 * Written starting in 2024 by contributors (see CREDITS.txt at repository's root)
 * To the extent possible under law, the author(s) have dedicated all copyright and related and neighboring rights to this software to the public domain worldwide. This software is distributed without any warranty.
 * A copy of the Unlicense should have been supplied as COPYING.txt in this repository. Alternatively, you can find it at <https://unlicense.org/>.
 */

use core::fmt::Write;

use serde::de::Error;
use serde::ser::{
    SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant, SerializeTuple,
    SerializeTupleStruct, SerializeTupleVariant,
};
use serde::Serializer;

use crate::{
    datum_write_display_as_string, DatumAtom, DatumToken, DatumTokenType, DatumWriter,
    DatumWriterState,
};

use crate::serde::error;

/// Controls how the serializer does indentation and spacing.
///
/// _Added in 1.1.0._
#[non_exhaustive]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Style {
    /// Skips spaces after certain tokens which never need them.
    Minified,
    /// Spacing-only. Matches [DatumWriter]'s behaviour.
    SpacingOnly,
    /// Indented, fancy, pretty-printed.
    Indented,
}

/// Similar to [crate::serde::de::PlainDeserializer], this represents a plain serializer without any funny business.
/// Unlike that struct, full access is allowed, as there isn't much in the way of additional state.
///
/// _Added in 1.1.0._
pub struct PlainSerializer<'write> {
    pub target: &'write mut dyn Write,
    pub style: Style,
    pub writer: DatumWriter,
}

impl<'write> PlainSerializer<'write> {
    /// Creates a new serializer.
    pub fn new(target: &'write mut dyn Write, style: Style) -> Self {
        Self {
            target,
            style,
            writer: DatumWriter::default(),
        }
    }
    pub(crate) fn write_token(&mut self, token: DatumToken<&str>) -> error::Result<()> {
        if self.style == Style::Minified {
            let kind = token.token_type();
            if kind == DatumTokenType::ListStart || kind == DatumTokenType::ListEnd {
                self.writer.state = DatumWriterState::None;
            }
        }
        self.writer
            .write_token(self.target, &token)
            .map_err(|e| error::Error::custom(e))?;
        if self.style == Style::Minified {
            let kind = token.token_type();
            if kind == DatumTokenType::String || kind == DatumTokenType::ListEnd {
                self.writer.state = DatumWriterState::None;
            }
        }
        Ok(())
    }
    pub(crate) fn write_atom(&mut self, token: DatumAtom<&str>) -> error::Result<()> {
        self.writer
            .write_atom(self.target, &token)
            .map_err(|e| error::Error::custom(e))?;
        if self.style == Style::Minified {
            if let DatumAtom::String(_) = &token {
                self.writer.state = DatumWriterState::None;
            }
        }
        Ok(())
    }
    /// Indent control: Opened block
    /// Run after starting a list.
    pub(crate) fn fmt_open_block(&mut self) -> error::Result<()> {
        if self.style == Style::Indented {
            self.writer.indent += 1;
            self.writer
                .write_newline(self.target)
                .map_err(|e| error::Error::custom(e))?;
        }
        Ok(())
    }
    /// Indent control: Close block
    /// Run before ending a list.
    pub(crate) fn fmt_close_block(&mut self) -> error::Result<()> {
        if self.style == Style::Indented {
            self.writer.indent -= 1;
        }
        Ok(())
    }
    /// Indent control: Seq/Map newline
    /// Run after each seq/map element.
    pub(crate) fn fmt_seq_newline(&mut self) -> error::Result<()> {
        if self.style == Style::Indented {
            self.writer
                .write_newline(self.target)
                .map_err(|e| error::Error::custom(e))?;
        }
        Ok(())
    }
}

impl<'a> Serializer for &'a mut PlainSerializer<'_> {
    type Ok = ();
    type Error = error::Error;
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    // -- Option/Unit --
    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        let b: DatumAtom<&str> = DatumAtom::Nil;
        self.write_atom(b)
    }
    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        let mut b: DatumToken<&str> = DatumToken::ListStart(0);
        self.write_token(b)?;
        b = DatumToken::ListEnd(0);
        self.write_token(b)
    }
    // -- Enum --
    fn serialize_newtype_variant<T: serde::Serialize + ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        let mut b: DatumToken<&str> = DatumToken::ListStart(0);
        self.write_token(b)?;
        self.write_atom(DatumAtom::Symbol(variant))?;
        value.serialize(&mut NewtypeVariantSerializer(self))?;
        b = DatumToken::ListEnd(0);
        self.write_token(b)
    }
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        let b: DatumToken<&str> = DatumToken::ListStart(0);
        self.write_token(b)?;
        self.write_atom(DatumAtom::Symbol(variant))?;
        Ok(self)
    }
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        let b: DatumToken<&str> = DatumToken::ListStart(0);
        self.write_token(b)?;
        self.write_atom(DatumAtom::Symbol(variant))?;
        self.fmt_open_block()?;
        Ok(self)
    }
    // -- Struct --
    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        let b: DatumToken<&str> = DatumToken::ListStart(0);
        self.write_token(b)?;
        self.fmt_open_block()?;
        Ok(self)
    }
    // -- Seq/Map --
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        let b: DatumToken<&str> = DatumToken::ListStart(0);
        self.write_token(b)?;
        self.fmt_open_block()?;
        Ok(self)
    }
    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        let b: DatumToken<&str> = DatumToken::ListStart(0);
        self.write_token(b)?;
        Ok(self)
    }
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        let b: DatumToken<&str> = DatumToken::ListStart(0);
        self.write_token(b)?;
        self.fmt_open_block()?;
        Ok(self)
    }
    // -- String --
    fn collect_str<T: core::fmt::Display + ?Sized>(
        self,
        value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        self.writer
            .emit_whitespace(self.target, false)
            .map_err(|e| error::Error::custom(e))?;
        datum_write_display_as_string(self.target, value).map_err(|e| error::Error::custom(e))?;
        if self.style == Style::Minified {
            self.writer.state = DatumWriterState::None;
        } else {
            self.writer.state = DatumWriterState::AfterToken;
        }
        Ok(())
    }
    serializer_invariants!();
}

// -- Seqlikes --

impl<'a> SerializeSeq for &'a mut PlainSerializer<'_> {
    type Ok = ();
    type Error = error::Error;
    fn serialize_element<T: serde::Serialize + ?Sized>(
        &mut self,
        value: &T,
    ) -> Result<(), Self::Error> {
        value.serialize(&mut **self)?;
        self.fmt_seq_newline()
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.fmt_close_block()?;
        let b: DatumToken<&str> = DatumToken::ListEnd(0);
        self.write_token(b)
    }
}

// Tuples don't get indentation and per-element newlines.

impl<'a> SerializeTuple for &'a mut PlainSerializer<'_> {
    type Ok = ();
    type Error = error::Error;
    fn serialize_element<T: serde::Serialize + ?Sized>(
        &mut self,
        value: &T,
    ) -> Result<(), Self::Error> {
        value.serialize(&mut **self)
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        let b: DatumToken<&str> = DatumToken::ListEnd(0);
        self.write_token(b)
    }
}

impl<'a> SerializeTupleStruct for &'a mut PlainSerializer<'_> {
    type Ok = ();
    type Error = error::Error;
    fn serialize_field<T: serde::Serialize + ?Sized>(
        &mut self,
        value: &T,
    ) -> Result<(), Self::Error> {
        SerializeTuple::serialize_element(self, value)
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        SerializeTuple::end(self)
    }
}

impl<'a> SerializeTupleVariant for &'a mut PlainSerializer<'_> {
    type Ok = ();
    type Error = error::Error;
    fn serialize_field<T: serde::Serialize + ?Sized>(
        &mut self,
        value: &T,
    ) -> Result<(), Self::Error> {
        SerializeTuple::serialize_element(self, value)
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        SerializeTuple::end(self)
    }
}

// -- Maplikes --

impl<'a> SerializeMap for &'a mut PlainSerializer<'_> {
    type Ok = ();
    type Error = error::Error;
    fn serialize_key<T: serde::Serialize + ?Sized>(&mut self, key: &T) -> Result<(), Self::Error> {
        key.serialize(&mut **self)
    }
    fn serialize_value<T: serde::Serialize + ?Sized>(
        &mut self,
        value: &T,
    ) -> Result<(), Self::Error> {
        value.serialize(&mut **self)?;
        self.fmt_seq_newline()
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.fmt_close_block()?;
        let b: DatumToken<&str> = DatumToken::ListEnd(0);
        self.write_token(b)
    }
}

impl<'a> SerializeStruct for &'a mut PlainSerializer<'_> {
    type Ok = ();
    type Error = error::Error;
    fn serialize_field<T: serde::Serialize + ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        let b: DatumAtom<&str> = DatumAtom::Symbol(key);
        self.write_atom(b)?;
        value.serialize(&mut **self)?;
        self.fmt_seq_newline()
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.fmt_close_block()?;
        let b: DatumToken<&str> = DatumToken::ListEnd(0);
        self.write_token(b)
    }
}

impl<'a> SerializeStructVariant for &'a mut PlainSerializer<'_> {
    type Ok = ();
    type Error = error::Error;
    fn serialize_field<T: serde::Serialize + ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        SerializeStruct::serialize_field(self, key, value)
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        SerializeStruct::end(self)
    }
}

// -- Newtype Variant --

/// NewtypeVariantSerializer writes the value inside a newtype variant.
struct NewtypeVariantSerializer<'ser, 'write>(&'ser mut PlainSerializer<'write>);

impl NewtypeVariantSerializer<'_, '_> {
    fn write_atom(&mut self, token: DatumAtom<&str>) -> error::Result<()> {
        self.0.write_atom(token)
    }
}

impl<'a> Serializer for &'a mut NewtypeVariantSerializer<'_, '_> {
    type Ok = ();
    type Error = error::Error;
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    // -- Forward --
    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.0.serialize_none()
    }
    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        self.0.serialize_unit()
    }
    fn collect_str<T: core::fmt::Display + ?Sized>(
        self,
        value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        self.0.collect_str(value)
    }
    // -- Enum --
    fn serialize_newtype_variant<T: serde::Serialize + ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        self.write_atom(DatumAtom::Symbol(variant))?;
        // this itself is a newtype variant, so keep the chain going
        value.serialize(self)
    }
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        self.write_atom(DatumAtom::Symbol(variant))?;
        Ok(self)
    }
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        self.write_atom(DatumAtom::Symbol(variant))?;
        self.0.fmt_open_block()?;
        Ok(self)
    }
    // -- Struct --
    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        self.0.fmt_open_block()?;
        Ok(self)
    }
    // -- Seq/Map --
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        self.0.fmt_open_block()?;
        Ok(self)
    }
    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Ok(self)
    }
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        self.0.fmt_open_block()?;
        Ok(self)
    }
    serializer_invariants!();
}

// -- Seqlikes --

impl<'a> SerializeSeq for &'a mut NewtypeVariantSerializer<'_, '_> {
    type Ok = ();
    type Error = error::Error;
    fn serialize_element<T: serde::Serialize + ?Sized>(
        &mut self,
        value: &T,
    ) -> Result<(), Self::Error> {
        value.serialize(&mut **self)?;
        self.0.fmt_seq_newline()
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.0.fmt_close_block()
    }
}

// Tuples don't get indentation and per-element newlines.

impl<'a> SerializeTuple for &'a mut NewtypeVariantSerializer<'_, '_> {
    type Ok = ();
    type Error = error::Error;
    fn serialize_element<T: serde::Serialize + ?Sized>(
        &mut self,
        value: &T,
    ) -> Result<(), Self::Error> {
        value.serialize(&mut **self)
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a> SerializeTupleStruct for &'a mut NewtypeVariantSerializer<'_, '_> {
    type Ok = ();
    type Error = error::Error;
    fn serialize_field<T: serde::Serialize + ?Sized>(
        &mut self,
        value: &T,
    ) -> Result<(), Self::Error> {
        SerializeTuple::serialize_element(self, value)
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        SerializeTuple::end(self)
    }
}

impl<'a> SerializeTupleVariant for &'a mut NewtypeVariantSerializer<'_, '_> {
    type Ok = ();
    type Error = error::Error;
    fn serialize_field<T: serde::Serialize + ?Sized>(
        &mut self,
        value: &T,
    ) -> Result<(), Self::Error> {
        SerializeTuple::serialize_element(self, value)
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        SerializeTuple::end(self)
    }
}

// -- Maplikes --

impl<'a> SerializeMap for &'a mut NewtypeVariantSerializer<'_, '_> {
    type Ok = ();
    type Error = error::Error;
    fn serialize_key<T: serde::Serialize + ?Sized>(&mut self, key: &T) -> Result<(), Self::Error> {
        key.serialize(&mut **self)
    }
    fn serialize_value<T: serde::Serialize + ?Sized>(
        &mut self,
        value: &T,
    ) -> Result<(), Self::Error> {
        value.serialize(&mut **self)?;
        self.0.fmt_seq_newline()
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.0.fmt_close_block()
    }
}

impl<'a> SerializeStruct for &'a mut NewtypeVariantSerializer<'_, '_> {
    type Ok = ();
    type Error = error::Error;
    fn serialize_field<T: serde::Serialize + ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        let b: DatumAtom<&str> = DatumAtom::Symbol(key);
        self.0.write_atom(b)?;
        value.serialize(&mut **self)?;
        self.0.fmt_seq_newline()
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.0.fmt_close_block()
    }
}

impl<'a> SerializeStructVariant for &'a mut NewtypeVariantSerializer<'_, '_> {
    type Ok = ();
    type Error = error::Error;
    fn serialize_field<T: serde::Serialize + ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        SerializeStruct::serialize_field(self, key, value)
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        SerializeStruct::end(self)
    }
}
