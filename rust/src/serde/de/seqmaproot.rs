/*
 * datum-rs - Quick to implement S-expression format
 * Written starting in 2024 by contributors (see CREDITS.txt at repository's root)
 * To the extent possible under law, the author(s) have dedicated all copyright and related and neighboring rights to this software to the public domain worldwide. This software is distributed without any warranty.
 * A copy of the Unlicense should have been supplied as COPYING.txt in this repository. Alternatively, you can find it at <https://unlicense.org/>.
 */

use core::ops::Deref;

use serde::{
    de::{EnumAccess, MapAccess, SeqAccess, VariantAccess},
    forward_to_deserialize_any, Deserializer,
};

use crate::{serde::error, DatumResult, DatumToken};

use crate::serde::de::PlainDeserializer;

/// 'Document Root' deserializer.
///
/// This is intended to deserialize an entire document at once, assuming the document is a sequence or map.
///
/// _Added in 1.1.0._
pub struct RootDeserializer<'iterator, B: Default + Deref<Target = str>>(
    pub PlainDeserializer<'iterator, B>,
);

impl<'iterator, B: Default + Deref<Target = str>> RootDeserializer<'iterator, B> {
    /// Creates the Deserializer from an iterator.
    ///
    /// _Added in 1.2.0._
    pub fn from_iterator(
        iterator: &'iterator mut dyn Iterator<Item = DatumResult<DatumToken<B>>>,
    ) -> Self {
        Self(PlainDeserializer::from_iterator(iterator))
    }
}

impl<'de, 'a, B: Default + Deref<Target = str>> Deserializer<'de>
    for &'a mut RootDeserializer<'_, B>
{
    type Error = error::Error;
    fn deserialize_any<V: serde::de::Visitor<'de>>(self, visitor: V) -> error::Result<V::Value> {
        self.0.deserialize_any(visitor)
    }
    fn deserialize_u64<V: serde::de::Visitor<'de>>(self, visitor: V) -> error::Result<V::Value> {
        self.0.deserialize_u64(visitor)
    }
    fn deserialize_option<V: serde::de::Visitor<'de>>(
        self,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        if self.0.has_next_token()? {
            visitor.visit_some(self)
        } else {
            visitor.visit_none()
        }
    }
    fn deserialize_unit<V: serde::de::Visitor<'de>>(
        self,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.0.deserialize_unit(visitor)
    }
    fn deserialize_seq<V: serde::de::Visitor<'de>>(
        self,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        visitor.visit_seq(AccessWrapper(self))
    }
    fn deserialize_tuple<V: serde::de::Visitor<'de>>(
        self,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        visitor.visit_seq(TupleAccess(self, len))
    }
    fn deserialize_map<V: serde::de::Visitor<'de>>(
        self,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        visitor.visit_map(AccessWrapper(self))
    }
    fn deserialize_enum<V: serde::de::Visitor<'de>>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        visitor.visit_enum(AccessWrapper(self))
    }
    deserializer_invariants!();
    fn is_human_readable(&self) -> bool {
        true
    }
}

/// special case problem solver
struct TupleAccess<'a, 'iterator, B: Default + Deref<Target = str>>(
    &'a mut RootDeserializer<'iterator, B>,
    usize,
);

impl<'de, 'a, B: Default + Deref<Target = str>> SeqAccess<'de> for TupleAccess<'a, '_, B> {
    type Error = error::Error;
    fn next_element_seed<T: serde::de::DeserializeSeed<'de>>(
        &mut self,
        seed: T,
    ) -> error::Result<Option<T::Value>> {
        if self.1 > 0 {
            self.1 -= 1;
            seed.deserialize(&mut self.0 .0).map(Some)
        } else {
            Ok(None)
        }
    }
}

/// Hides access traits and also solves some weird lifetime problems.
struct AccessWrapper<'a, 'iterator, B: Default + Deref<Target = str>>(
    &'a mut RootDeserializer<'iterator, B>,
);

impl<'de, 'a, B: Default + Deref<Target = str>> SeqAccess<'de> for AccessWrapper<'a, '_, B> {
    type Error = error::Error;
    fn next_element_seed<T: serde::de::DeserializeSeed<'de>>(
        &mut self,
        seed: T,
    ) -> error::Result<Option<T::Value>> {
        if self.0 .0.has_next_token()? {
            seed.deserialize(&mut self.0 .0).map(Some)
        } else {
            Ok(None)
        }
    }
}

impl<'de, 'a, B: Default + Deref<Target = str>> MapAccess<'de> for AccessWrapper<'a, '_, B> {
    type Error = error::Error;
    fn next_key_seed<T: serde::de::DeserializeSeed<'de>>(
        &mut self,
        seed: T,
    ) -> error::Result<Option<T::Value>> {
        if self.0 .0.has_next_token()? {
            seed.deserialize(&mut self.0 .0).map(Some)
        } else {
            Ok(None)
        }
    }
    fn next_value_seed<V: serde::de::DeserializeSeed<'de>>(
        &mut self,
        seed: V,
    ) -> error::Result<V::Value> {
        seed.deserialize(&mut self.0 .0)
    }
}

impl<'de, 'a, B: Default + Deref<Target = str>> EnumAccess<'de> for AccessWrapper<'a, '_, B> {
    type Error = error::Error;
    type Variant = Self;
    fn variant_seed<V: serde::de::DeserializeSeed<'de>>(
        self,
        seed: V,
    ) -> Result<(V::Value, Self::Variant), Self::Error> {
        Ok((seed.deserialize(&mut self.0 .0)?, self))
    }
}

impl<'de, 'a, B: Default + Deref<Target = str>> VariantAccess<'de> for AccessWrapper<'a, '_, B> {
    type Error = error::Error;
    fn unit_variant(self) -> Result<(), Self::Error> {
        Ok(())
    }
    fn newtype_variant_seed<T: serde::de::DeserializeSeed<'de>>(
        self,
        seed: T,
    ) -> Result<T::Value, Self::Error> {
        // Critically important that this is at root level.
        seed.deserialize(&mut *self.0)
    }
    fn tuple_variant<V: serde::de::Visitor<'de>>(
        self,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        visitor.visit_seq(self)
    }
    fn struct_variant<V: serde::de::Visitor<'de>>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        visitor.visit_map(self)
    }
}
