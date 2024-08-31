/*
 * datum-rs - Quick to implement S-expression format
 * Written starting in 2024 by contributors (see CREDITS.txt at repository's root)
 * To the extent possible under law, the author(s) have dedicated all copyright and related and neighboring rights to this software to the public domain worldwide. This software is distributed without any warranty.
 * A copy of the Unlicense should have been supplied as COPYING.txt in this repository. Alternatively, you can find it at <https://unlicense.org/>.
 */

// This project is for testing the possibility of Serde support in Datum.

use std::{convert::TryFrom, ops::Deref};

use datum::{datum_char_to_token_pipeline, DatumAtom, DatumResult, DatumToken, IntoViaDatumPipe};
use serde::{de::{EnumAccess, Error, IntoDeserializer, MapAccess, SeqAccess, VariantAccess}, forward_to_deserialize_any, Deserialize, Deserializer};

/// A 'plain' deserializer.
/// No fancy formatting; will expect values in sequence, fails on EOF.
pub struct PlainDeserializer<B: Default + Deref<Target = str>, I: Iterator<Item = DatumResult<DatumToken<B>>>> {
    iterator: I,
    hold: Option<DatumToken<B>>
}

impl<B: Default + Deref<Target = str>, I: Iterator<Item = DatumResult<DatumToken<B>>>> PlainDeserializer<B, I> {
    /// Creates the Deserializer from an iterator.
    pub fn from_iterator(iterator: I) -> Self {
        Self {
            iterator,
            hold: None
        }
    }
    /// Checks if a next token exists.
    /// Errors indicate non-EOF errors.
    pub fn has_next_token(&mut self) -> Result<bool, serde::de::value::Error> {
        if self.hold.is_some() {
            Ok(true)
        } else {
            let res = self.iterator.next();
            if let Some(v) = res {
                self.hold = Some(v.map_err(|e| serde::de::value::Error::custom(e))?);
                Ok(true)
            } else {
                Ok(false)
            }
        }
    }
    /// Retrieves the next token and handles error casting.
    fn next_token(&mut self) -> Result<DatumToken<B>, serde::de::value::Error> {
        if let Some(token) = self.hold.take() {
            Ok(token)
        } else {
            let res = self.iterator.next();
            if let Some(v) = res {
                v.map_err(|e| serde::de::value::Error::custom(e))
            } else {
                Err(serde::de::value::Error::custom("EOF?"))
            }
        }
    }
}

impl<'de, 'a, B: Default + Deref<Target = str>, I: Iterator<Item = DatumResult<DatumToken<B>>>> SeqAccess<'de> for PlainDeserializer<B, I> {
    type Error = serde::de::value::Error;
    fn next_element_seed<T: serde::de::DeserializeSeed<'de>>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error> {
        let token = self.next_token()?;
        if let DatumToken::ListEnd(_) = token {
            Ok(None)
        } else {
            self.hold = Some(token);
            seed.deserialize(self).map(Some)
        }
    }
}

impl<'de, 'a, B: Default + Deref<Target = str>, I: Iterator<Item = DatumResult<DatumToken<B>>>> MapAccess<'de> for PlainDeserializer<B, I> {
    type Error = serde::de::value::Error;
    fn next_key_seed<K: serde::de::DeserializeSeed<'de>>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error> {
        self.next_element_seed(seed)
    }
    fn next_value_seed<V: serde::de::DeserializeSeed<'de>>(&mut self, seed: V) -> Result<V::Value, Self::Error> {
        seed.deserialize(self)
    }
}

struct Enum<'a, B: Default + Deref<Target = str>, I: Iterator<Item = DatumResult<DatumToken<B>>>>(&'a mut PlainDeserializer<B, I>);

impl<'de, 'a, B: Default + Deref<Target = str>, I: Iterator<Item = DatumResult<DatumToken<B>>>> EnumAccess<'de> for Enum<'a, B, I> {
    type Error = serde::de::value::Error;
    type Variant = Self;
    fn variant_seed<V: serde::de::DeserializeSeed<'de>>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error> {
        Ok((seed.deserialize(&mut *self.0)?, self))
    }
}

impl<'de, 'a, B: Default + Deref<Target = str>, I: Iterator<Item = DatumResult<DatumToken<B>>>> VariantAccess<'de> for Enum<'a, B, I> {
    type Error = serde::de::value::Error;
    fn unit_variant(self) -> Result<(), Self::Error> {
        if let DatumToken::ListEnd(_) = self.0.next_token()? {
            Ok(())
        } else {
            Err(serde::de::value::Error::custom("unit variants represented as (Variant) must be properly contained"))
        }
    }
    fn newtype_variant_seed<T: serde::de::DeserializeSeed<'de>>(self, seed: T) -> Result<T::Value, Self::Error> {
        let res = seed.deserialize(&mut *self.0)?;
        // workaround to catch the ')'
        if let DatumToken::ListEnd(_) = self.0.next_token()? {
            Ok(res)
        } else {
            Err(serde::de::value::Error::custom("newtype variants must be properly contained"))
        }
    }
    fn tuple_variant<V: serde::de::Visitor<'de>>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_seq(self.0)
    }
    fn struct_variant<V: serde::de::Visitor<'de>>(self, _fields: &'static [&'static str], visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_map(self.0)
    }
}

impl<'de, 'a, B: Default + Deref<Target = str>, I: Iterator<Item = DatumResult<DatumToken<B>>>> Deserializer<'de> for &'a mut PlainDeserializer<B, I> {
    type Error = serde::de::value::Error;
    // deserialize_any itself
    fn deserialize_any<V: serde::de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let token = self.next_token()?;
        if let DatumToken::ListStart(_) = token {
            // consume the list start and let the SeqAccess impl. take care of the rest
            visitor.visit_seq(self)
        } else if let DatumToken::ListEnd(_) = token {
            Err(serde::de::value::Error::custom("unexpected list end"))
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
                Err(err) => Err(serde::de::value::Error::custom(err))
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
        let token = self.next_token()?;
        match token {
            DatumToken::Symbol(_, text) => visitor.visit_enum(text.into_deserializer()),
            DatumToken::ListStart(_) => visitor.visit_enum(Enum(self)),
            _ => Err(serde::de::value::Error::custom("expected symbol or list-with-variant, got something else"))
        }
    }
    fn deserialize_option<V: serde::de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let token = self.next_token()?;
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
    fn deserialize_map<V: serde::de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        match self.next_token()? {
            DatumToken::ListStart(_) => visitor.visit_map(self),
            _ => Err(serde::de::value::Error::custom("expected map, got something else"))
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
    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string identifier
        bytes byte_buf unit seq tuple tuple_struct
        unit_struct ignored_any
    }
}

/// Sequence-root deserializer.
/// This treats the file as being a sequence of values, and then whatever comes up in the file is treated as elements of that sequence.
/// In practice, this and [MapRootDeserializer] are the two 'canonical' Datum document forms.
pub struct SeqRootDeserializer<B: Default + Deref<Target = str>, I: Iterator<Item = DatumResult<DatumToken<B>>>>(pub PlainDeserializer<B, I>);

impl<'de, 'a, B: Default + Deref<Target = str>, I: Iterator<Item = DatumResult<DatumToken<B>>>> SeqAccess<'de> for &'a mut SeqRootDeserializer<B, I> {
    type Error = serde::de::value::Error;
    fn next_element_seed<T: serde::de::DeserializeSeed<'de>>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error> {
        if self.0.has_next_token()? {
            seed.deserialize(&mut self.0).map(Some)
        } else {
            Ok(None)
        }
    }
}

impl<'de, 'a, B: Default + Deref<Target = str>, I: Iterator<Item = DatumResult<DatumToken<B>>>> Deserializer<'de> for &'a mut SeqRootDeserializer<B, I> {
    type Error = serde::de::value::Error;
    fn deserialize_any<V: serde::de::Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_seq(self)
    }
    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string identifier
        bytes byte_buf unit seq tuple tuple_struct option newtype_struct map struct enum
        unit_struct ignored_any
    }
}

#[derive(Deserialize, Debug)]
enum MyEnum {
    Apple,
    Berry
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
enum MyComplexEnum {
    Stuff,
    ThingCount(i32)
}


#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct MyExampleType {
    pub wobble: i32,
    pub myenum: MyEnum,
    pub myvec: Vec<MyComplexEnum>
}

type MyExampleDocument = Vec<MyExampleType>;

fn main() {
    let mut tmp = PlainDeserializer::from_iterator(include_str!("../example.scm").chars().via_datum_pipe(datum_char_to_token_pipeline()));
    let vec: MyExampleDocument = MyExampleDocument::deserialize(&mut tmp).unwrap();
    for v in vec {
        println!("{:?}", v);
    }
}
