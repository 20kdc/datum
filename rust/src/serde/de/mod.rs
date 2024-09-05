/*
 * datum-rs - Quick to implement S-expression format
 * Written starting in 2024 by contributors (see CREDITS.txt at repository's root)
 * To the extent possible under law, the author(s) have dedicated all copyright and related and neighboring rights to this software to the public domain worldwide. This software is distributed without any warranty.
 * A copy of the Unlicense should have been supplied as COPYING.txt in this repository. Alternatively, you can find it at <https://unlicense.org/>.
 */

//! [serde::Deserializer] implementation and supporting types.
//!
//! _Added in 1.1.0._

/// Deserializer common behaviour.
macro_rules! deserializer_invariants {
    () => {
        forward_to_deserialize_any! {
            bool i8 i16 i32 i64 i128 u8 u16 u32 u128 f32 f64 char
            str string identifier
            ignored_any
            bytes byte_buf
        }
        fn deserialize_newtype_struct<V: serde::de::Visitor<'de>>(
            self,
            _name: &'static str,
            visitor: V,
        ) -> Result<V::Value, Self::Error> {
            visitor.visit_newtype_struct(self)
        }
        fn deserialize_struct<V: serde::de::Visitor<'de>>(
            self,
            _name: &'static str,
            _fields: &'static [&'static str],
            visitor: V,
        ) -> Result<V::Value, Self::Error> {
            self.deserialize_map(visitor)
        }
        fn deserialize_unit_struct<V: serde::de::Visitor<'de>>(
            self,
            _name: &'static str,
            visitor: V,
        ) -> Result<V::Value, Self::Error> {
            self.deserialize_unit(visitor)
        }
        fn deserialize_tuple_struct<V: serde::de::Visitor<'de>>(
            self,
            _name: &'static str,
            len: usize,
            visitor: V,
        ) -> Result<V::Value, Self::Error> {
            self.deserialize_tuple(len, visitor)
        }
    };
}

mod deserializer;
pub use deserializer::*;
mod seqmaproot;
pub use seqmaproot::*;
