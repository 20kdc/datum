/*
 * datum-rs - Quick to implement S-expression format
 * Written starting in 2024 by contributors (see CREDITS.txt at repository's root)
 * To the extent possible under law, the author(s) have dedicated all copyright and related and neighboring rights to this software to the public domain worldwide. This software is distributed without any warranty.
 * A copy of the Unlicense should have been supplied as COPYING.txt in this repository. Alternatively, you can find it at <https://unlicense.org/>.
 */

//! [serde::Serializer] implementation and supporting types.
//!
//! _Added in 1.1.0._

/// Serializer common behaviour.
macro_rules! serializer_invariants {
    () => {
        fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> {
            Err(error::Error::custom(
                "Byte arrays cannot be serialized at present.",
            ))
        }
        fn serialize_newtype_struct<T: serde::Serialize + ?Sized>(
            self,
            _name: &'static str,
            value: &T,
        ) -> Result<Self::Ok, Self::Error> {
            value.serialize(self)
        }
        fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
            self.serialize_f64(v as f64)
        }
        fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
            self.serialize_i64(v as i64)
        }
        fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
            self.serialize_i64(v as i64)
        }
        fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
            self.serialize_i64(v as i64)
        }
        fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
            self.serialize_i64(v as i64)
        }
        fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
            self.serialize_i64(v as i64)
        }
        fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
            self.serialize_i64(v as i64)
        }
        fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
            self.serialize_i64(v as i64)
        }
        fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
            self.collect_str(&v)
        }
        fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
            self.serialize_unit()
        }
        fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
            let b: DatumAtom<&str> = DatumAtom::Boolean(v);
            self.write_atom(b)
        }
        fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
            let b: DatumAtom<&str> = DatumAtom::Integer(v);
            self.write_atom(b)
        }
        fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
            let b: DatumAtom<&str> = DatumAtom::Float(v);
            self.write_atom(b)
        }
        fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
            let b: DatumAtom<&str> = DatumAtom::String(v);
            self.write_atom(b)
        }
        fn serialize_unit_variant(
            self,
            _name: &'static str,
            _variant_index: u32,
            variant: &'static str,
        ) -> Result<Self::Ok, Self::Error> {
            self.write_atom(DatumAtom::Symbol(variant))
        }
        fn serialize_some<T: serde::Serialize + ?Sized>(
            self,
            value: &T,
        ) -> Result<Self::Ok, Self::Error> {
            value.serialize(self)
        }
        type SerializeTupleStruct = Self::SerializeTuple;
        fn serialize_tuple_struct(
            self,
            _name: &'static str,
            len: usize,
        ) -> Result<Self::SerializeTupleStruct, Self::Error> {
            self.serialize_tuple(len)
        }
    };
}

mod serializer;
pub use serializer::*;
mod seqmaproot;
pub use seqmaproot::*;
