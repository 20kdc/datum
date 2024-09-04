/*
 * datum-rs - Quick to implement S-expression format
 * Written starting in 2024 by contributors (see CREDITS.txt at repository's root)
 * To the extent possible under law, the author(s) have dedicated all copyright and related and neighboring rights to this software to the public domain worldwide. This software is distributed without any warranty.
 * A copy of the Unlicense should have been supplied as COPYING.txt in this repository. Alternatively, you can find it at <https://unlicense.org/>.
 */

use std::{convert::TryFrom, ops::Deref};

use crate::{datum_error, DatumAtom, DatumError, DatumOffset, DatumResult, DatumToken};
use serde::{
    de::{EnumAccess, IntoDeserializer, MapAccess, SeqAccess, VariantAccess},
    forward_to_deserialize_any, Deserializer,
};

use crate::serde::error;
use crate::serde::error::error_from_datum;

/// A 'plain' deserializer.
/// Expects values in sequence, fails on EOF.
pub struct PlainDeserializer<'iterator, B: Default + Deref<Target = str>> {
    iterator: &'iterator mut dyn Iterator<Item = DatumResult<DatumToken<B>>>,
    hold: Option<DatumToken<B>>,
    last_seen_offset: DatumOffset,
}

impl<'iterator, B: Default + Deref<Target = str>> PlainDeserializer<'iterator, B> {
    /// Creates the Deserializer from an iterator.
    pub fn from_iterator(
        iterator: &'iterator mut dyn Iterator<Item = DatumResult<DatumToken<B>>>,
    ) -> Self {
        Self {
            iterator,
            hold: None,
            last_seen_offset: 0,
        }
    }
    /// Checks if a next token exists.
    /// Errors indicate non-EOF errors.
    pub fn has_next_token(&mut self) -> error::Result<bool> {
        if self.hold.is_some() {
            Ok(true)
        } else {
            let res = self.iterator.next();
            if let Some(v) = res {
                self.hold = Some(v.map_err(error_from_datum)?);
                Ok(true)
            } else {
                Ok(false)
            }
        }
    }
    /// Retrieves the next token and handles error casting.
    fn next_token(&mut self, eof_error: DatumError) -> error::Result<DatumToken<B>> {
        if let Some(token) = self.hold.take() {
            Ok(token)
        } else {
            let res = self.iterator.next();
            if let Some(v) = res {
                if let Ok(tkn) = &v {
                    self.last_seen_offset = tkn.offset();
                }
                v.map_err(error_from_datum)
            } else {
                Err(error_from_datum(eof_error))
            }
        }
    }
    /// Expects a list end.
    fn expect_list_end(&mut self) -> error::Result<()> {
        if let DatumToken::ListEnd(_) = self.next_token(datum_error!(
            Interrupted,
            self.last_seen_offset,
            "unexpected EOF, expected list end"
        ))? {
            Ok(())
        } else {
            Err(error_from_datum(datum_error!(
                BadData,
                self.last_seen_offset,
                "expected list end and got something else"
            )))
        }
    }
}

/// Hides access traits and also solves some weird lifetime problems.
struct AccessWrapper<'a, 'iterator, B: Default + Deref<Target = str>>(
    &'a mut PlainDeserializer<'iterator, B>,
);

impl<'de, 'a, B: Default + Deref<Target = str>> SeqAccess<'de> for AccessWrapper<'a, '_, B> {
    type Error = error::Error;
    fn next_element_seed<T: serde::de::DeserializeSeed<'de>>(
        &mut self,
        seed: T,
    ) -> Result<Option<T::Value>, Self::Error> {
        let token = self.0.next_token(datum_error!(
            Interrupted,
            self.0.last_seen_offset,
            "seq: unexpected EOF, expected next element or list end"
        ))?;
        if let DatumToken::ListEnd(_) = token {
            // Ok, so, here's a sneaky thing that Serde does which is ?undocumented? outside of the JSON example?
            // You CANNOT rely on the sequence accessor being fully consumed.
            // You MUST have some check after calling [Visitor::visit_seq].
            self.0.hold = Some(token);
            Ok(None)
        } else {
            self.0.hold = Some(token);
            seed.deserialize(&mut *self.0).map(Some)
        }
    }
}

impl<'de, 'a, B: Default + Deref<Target = str>> MapAccess<'de> for AccessWrapper<'a, '_, B> {
    type Error = error::Error;
    fn next_key_seed<K: serde::de::DeserializeSeed<'de>>(
        &mut self,
        seed: K,
    ) -> Result<Option<K::Value>, Self::Error> {
        self.next_element_seed(seed)
    }
    fn next_value_seed<V: serde::de::DeserializeSeed<'de>>(
        &mut self,
        seed: V,
    ) -> Result<V::Value, Self::Error> {
        seed.deserialize(&mut *self.0)
    }
}

impl<'de, 'a, B: Default + Deref<Target = str>> EnumAccess<'de> for AccessWrapper<'a, '_, B> {
    type Error = error::Error;
    type Variant = Self;
    fn variant_seed<V: serde::de::DeserializeSeed<'de>>(
        self,
        seed: V,
    ) -> Result<(V::Value, Self::Variant), Self::Error> {
        Ok((seed.deserialize(&mut *self.0)?, self))
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

impl<'de, 'a, B: Default + Deref<Target = str>> Deserializer<'de>
    for &'a mut PlainDeserializer<'_, B>
{
    type Error = error::Error;
    // deserialize_any itself
    fn deserialize_any<V: serde::de::Visitor<'de>>(
        self,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        let token = self.next_token(datum_error!(
            Interrupted,
            self.last_seen_offset,
            "any: Unexpected EOF, expected value"
        ))?;
        if let DatumToken::ListStart(_) = token {
            // consume the list start and let the SeqAccess impl. take care of the rest
            let res = visitor.visit_seq(AccessWrapper(self))?;
            self.expect_list_end()?;
            Ok(res)
        } else if let DatumToken::ListEnd(_) = token {
            Err(error_from_datum(datum_error!(
                BadData,
                self.last_seen_offset,
                "any: unexpected list end"
            )))
        } else {
            match DatumAtom::try_from(token) {
                Ok(atom) => match atom {
                    DatumAtom::String(b) => visitor.visit_str(&b),
                    DatumAtom::Symbol(b) => visitor.visit_str(&b),
                    DatumAtom::Integer(i) => visitor.visit_i64(i),
                    DatumAtom::Float(v) => visitor.visit_f64(v),
                    DatumAtom::Boolean(v) => visitor.visit_bool(v),
                    // Nil is used for unit because using an empty list feels like it'd be weird interop-wise.
                    // Also the example did it.
                    DatumAtom::Nil => visitor.visit_unit(),
                },
                Err(err) => Err(error_from_datum(err)),
            }
        }
    }
    fn is_human_readable(&self) -> bool {
        true
    }
    // -- special handling --
    fn deserialize_enum<V: serde::de::Visitor<'de>>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        let token = self.next_token(datum_error!(
            Interrupted,
            self.last_seen_offset,
            "enum: unexpected EOF, expected value"
        ))?;
        match token {
            DatumToken::Symbol(_, text) => visitor.visit_enum(text.into_deserializer()),
            DatumToken::ListStart(_) => {
                let res = visitor.visit_enum(AccessWrapper(self))?;
                self.expect_list_end()?;
                Ok(res)
            }
            _ => Err(error_from_datum(datum_error!(
                BadData,
                self.last_seen_offset,
                "enum: expected symbol or list-with-variant, got something else"
            ))),
        }
    }
    fn deserialize_option<V: serde::de::Visitor<'de>>(
        self,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        let token = self.next_token(datum_error!(
            Interrupted,
            self.last_seen_offset,
            "enum: unexpected EOF, expected value"
        ))?;
        match &token {
            DatumToken::SpecialID(_, v) => {
                if v.eq_ignore_ascii_case("nil") {
                    return visitor.visit_none();
                }
            }
            _ => {}
        }
        self.hold = Some(token);
        visitor.visit_some(self)
    }
    fn deserialize_unit<V: serde::de::Visitor<'de>>(
        self,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        let token = self.next_token(datum_error!(
            Interrupted,
            self.last_seen_offset,
            "unit: unexpected EOF, expected value"
        ))?;
        match &token {
            DatumToken::ListStart(_) => {
                self.expect_list_end()?;
                visitor.visit_unit()
            }
            _ => {
                self.hold = Some(token);
                self.deserialize_any(visitor)
            }
        }
    }
    fn deserialize_map<V: serde::de::Visitor<'de>>(
        self,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        match self.next_token(datum_error!(
            Interrupted,
            self.last_seen_offset,
            "map: unexpected EOF, expected list"
        ))? {
            DatumToken::ListStart(_) => {
                let res = visitor.visit_map(AccessWrapper(self))?;
                self.expect_list_end()?;
                Ok(res)
            }
            _ => Err(error_from_datum(datum_error!(
                BadData,
                self.last_seen_offset,
                "map: expected list, got something else"
            ))),
        }
    }
    // -- forwarders/simple type aliases --
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
    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char
        str string identifier
        seq tuple tuple_struct
        ignored_any
        bytes byte_buf
    }
}
