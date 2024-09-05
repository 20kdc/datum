/*
 * datum-rs - Quick to implement S-expression format
 * Written starting in 2024 by contributors (see CREDITS.txt at repository's root)
 * To the extent possible under law, the author(s) have dedicated all copyright and related and neighboring rights to this software to the public domain worldwide. This software is distributed without any warranty.
 * A copy of the Unlicense should have been supplied as COPYING.txt in this repository. Alternatively, you can find it at <https://unlicense.org/>.
 */

use std::ops::Deref;

use serde::{
    de::{MapAccess, SeqAccess},
    forward_to_deserialize_any, Deserializer,
};

use crate::serde::error;

use crate::serde::de::PlainDeserializer;

/// Sequence-root deserializer.
/// This treats the file as being a sequence of values, and then whatever comes up in the file is treated as elements of that sequence.
/// In practice, this and [MapRootDeserializer] are the two 'canonical' Datum document forms.
///
/// _Added in 1.1.0._
pub struct SeqRootDeserializer<'iterator, B: Default + Deref<Target = str>>(
    pub PlainDeserializer<'iterator, B>,
);

impl<'de, 'a, B: Default + Deref<Target = str>> SeqAccess<'de>
    for &'a mut SeqRootDeserializer<'_, B>
{
    type Error = error::Error;
    fn next_element_seed<T: serde::de::DeserializeSeed<'de>>(
        &mut self,
        seed: T,
    ) -> error::Result<Option<T::Value>> {
        if self.0.has_next_token()? {
            seed.deserialize(&mut self.0).map(Some)
        } else {
            Ok(None)
        }
    }
}

impl<'de, 'a, B: Default + Deref<Target = str>> Deserializer<'de>
    for &'a mut SeqRootDeserializer<'_, B>
{
    type Error = error::Error;
    fn deserialize_any<V: serde::de::Visitor<'de>>(self, visitor: V) -> error::Result<V::Value> {
        visitor.visit_seq(self)
    }
    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string identifier
        bytes byte_buf unit seq tuple tuple_struct option newtype_struct map struct enum
        unit_struct ignored_any
    }
}

/// Map-root deserializer.
/// This treats the file as being a sequence of map entry pairs.
/// In practice, this and [SeqRootDeserializer] are the two 'canonical' Datum document forms.
///
/// _Added in 1.1.0._
pub struct MapRootDeserializer<'iterator, B: Default + Deref<Target = str>>(
    pub PlainDeserializer<'iterator, B>,
);

impl<'de, 'a, B: Default + Deref<Target = str>> MapAccess<'de>
    for &'a mut MapRootDeserializer<'_, B>
{
    type Error = error::Error;
    fn next_key_seed<T: serde::de::DeserializeSeed<'de>>(
        &mut self,
        seed: T,
    ) -> error::Result<Option<T::Value>> {
        if self.0.has_next_token()? {
            seed.deserialize(&mut self.0).map(Some)
        } else {
            Ok(None)
        }
    }
    fn next_value_seed<V: serde::de::DeserializeSeed<'de>>(
        &mut self,
        seed: V,
    ) -> error::Result<V::Value> {
        seed.deserialize(&mut self.0)
    }
}

impl<'de, 'a, B: Default + Deref<Target = str>> Deserializer<'de>
    for &'a mut MapRootDeserializer<'_, B>
{
    type Error = error::Error;
    fn deserialize_any<V: serde::de::Visitor<'de>>(self, visitor: V) -> error::Result<V::Value> {
        visitor.visit_map(self)
    }
    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string identifier
        bytes byte_buf unit seq tuple tuple_struct option newtype_struct map struct enum
        unit_struct ignored_any
    }
}
