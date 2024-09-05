/*
 * datum-rs - Quick to implement S-expression format
 * Written starting in 2024 by contributors (see CREDITS.txt at repository's root)
 * To the extent possible under law, the author(s) have dedicated all copyright and related and neighboring rights to this software to the public domain worldwide. This software is distributed without any warranty.
 * A copy of the Unlicense should have been supplied as COPYING.txt in this repository. Alternatively, you can find it at <https://unlicense.org/>.
 */

use serde::de::Error;
use serde::ser::{
    SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant, SerializeTuple,
    SerializeTupleStruct, SerializeTupleVariant,
};
use serde::Serializer;

use crate::DatumAtom;

use crate::serde::error;

use super::PlainSerializer;

/// [RootSerializer] serializes a document as a root-level sequence or map.
/// This matches [crate::serde::de::SeqRootDeserializer] and [crate::serde::de::MapRootDeserializer].
///
/// _Added in 1.1.0._
pub struct RootSerializer<'write>(pub PlainSerializer<'write>);

macro_rules! type_forward {
    ($fn_name: ident, $type: ty) => {
        fn $fn_name(self, v: $type) -> error::Result<()> {
            self.0.$fn_name(v)?;
            self.0.fmt_seq_newline()
        }
    };
}

impl<'a> Serializer for &'a mut RootSerializer<'_> {
    type Ok = ();
    type Error = error::Error;
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    // -- Trivial --
    type_forward!(serialize_bool, bool);
    type_forward!(serialize_i8, i8);
    type_forward!(serialize_i16, i16);
    type_forward!(serialize_i32, i32);
    type_forward!(serialize_i64, i64);
    type_forward!(serialize_u8, u8);
    type_forward!(serialize_u16, u16);
    type_forward!(serialize_u32, u32);
    type_forward!(serialize_u64, u64);
    type_forward!(serialize_f32, f32);
    type_forward!(serialize_f64, f64);
    type_forward!(serialize_char, char);
    type_forward!(serialize_str, &str);
    type_forward!(serialize_bytes, &[u8]);
    type_forward!(serialize_unit_struct, &'static str);
    fn collect_str<T: core::fmt::Display + ?Sized>(
        self,
        value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        self.0.collect_str(value)?;
        self.0.fmt_seq_newline()
    }
    // -- Key Aliases --
    fn serialize_newtype_struct<T: serde::Serialize + ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        value.serialize(self)
    }
    // -- Option/Unit --
    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        self.0.serialize_unit()?;
        self.0.fmt_seq_newline()
    }
    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(error::Error::custom("not a supported type for datum's RootSerializer"))
    }
    fn serialize_some<T: serde::Serialize + ?Sized>(
        self,
        value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        value.serialize(self)
    }
    // -- Enum --
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.0.write_atom(DatumAtom::Symbol(variant))
    }
    fn serialize_newtype_variant<T: serde::Serialize + ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        self.0.write_atom(DatumAtom::Symbol(variant))?;
        self.0.fmt_seq_newline()?;
        value.serialize(self)
    }
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        self.0.write_atom(DatumAtom::Symbol(variant))?;
        self.0.fmt_seq_newline()?;
        Ok(self)
    }
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        self.0.write_atom(DatumAtom::Symbol(variant))?;
        self.0.fmt_seq_newline()?;
        Ok(self)
    }
    // -- Struct --
    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(self)
    }
    // -- Seq/Map --
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(self)
    }
    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Ok(self)
    }
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Ok(self)
    }
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(self)
    }
}

// -- Seqlikes --

impl<'a> SerializeSeq for &'a mut RootSerializer<'_> {
    type Ok = ();
    type Error = error::Error;
    fn serialize_element<T: serde::Serialize + ?Sized>(
        &mut self,
        value: &T,
    ) -> Result<(), Self::Error> {
        value.serialize(&mut self.0)?;
        self.0.fmt_seq_newline()
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

// Tuples don't get indentation and per-element newlines.

impl<'a> SerializeTuple for &'a mut RootSerializer<'_> {
    type Ok = ();
    type Error = error::Error;
    fn serialize_element<T: serde::Serialize + ?Sized>(
        &mut self,
        value: &T,
    ) -> Result<(), Self::Error> {
        SerializeSeq::serialize_element(self, value)
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        SerializeSeq::end(self)
    }
}

impl<'a> SerializeTupleStruct for &'a mut RootSerializer<'_> {
    type Ok = ();
    type Error = error::Error;
    fn serialize_field<T: serde::Serialize + ?Sized>(
        &mut self,
        value: &T,
    ) -> Result<(), Self::Error> {
        SerializeSeq::serialize_element(self, value)
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        SerializeSeq::end(self)
    }
}

impl<'a> SerializeTupleVariant for &'a mut RootSerializer<'_> {
    type Ok = ();
    type Error = error::Error;
    fn serialize_field<T: serde::Serialize + ?Sized>(
        &mut self,
        value: &T,
    ) -> Result<(), Self::Error> {
        SerializeSeq::serialize_element(self, value)
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        SerializeSeq::end(self)
    }
}

// -- Maplikes --

impl<'a> SerializeMap for &'a mut RootSerializer<'_> {
    type Ok = ();
    type Error = error::Error;
    fn serialize_key<T: serde::Serialize + ?Sized>(&mut self, key: &T) -> Result<(), Self::Error> {
        key.serialize(&mut self.0)
    }
    fn serialize_value<T: serde::Serialize + ?Sized>(
        &mut self,
        value: &T,
    ) -> Result<(), Self::Error> {
        value.serialize(&mut self.0)?;
        self.0.fmt_seq_newline()
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a> SerializeStruct for &'a mut RootSerializer<'_> {
    type Ok = ();
    type Error = error::Error;
    fn serialize_field<T: serde::Serialize + ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        let b: DatumAtom<&str> = DatumAtom::Symbol(key);
        self.0.write_atom(b)?;
        value.serialize(&mut self.0)?;
        self.0.fmt_seq_newline()
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a> SerializeStructVariant for &'a mut RootSerializer<'_> {
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
