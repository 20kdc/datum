/*
 * datum-rs - Quick to implement S-expression format
 * Written starting in 2024 by contributors (see CREDITS.txt at repository's root)
 * To the extent possible under law, the author(s) have dedicated all copyright and related and neighboring rights to this software to the public domain worldwide. This software is distributed without any warranty.
 * A copy of the Unlicense should have been supplied as COPYING.txt in this repository. Alternatively, you can find it at <https://unlicense.org/>.
 */

//! Serde serializer/deserializer implementation and supporting types.
//!
//! _Added in 1.1.0._

use core::{fmt::Write, ops::Deref};

use serde::{Deserialize, Serialize};

use crate::{DatumResult, DatumToken};

pub mod error {
    use crate::DatumError;

    /// Error type.
    /// This was going to be a custom type, but it turns out that doing this is hilariously dangerous due to:
    ///
    /// * `serde::de::Error` is bounded on if-and-only-if Serde's `std` feature is enabled (IMO this is a violation of the positive feature principle)
    /// * This bound cannot be fulfilled in `no_std` contexts without unstable features
    ///
    /// _Added in 1.1.0._
    pub type Error = serde::de::value::Error;
    /// Result type, nothing special here.
    ///
    /// _Added in 1.1.0._
    pub type Result<T> = core::result::Result<T, Error>;
    /// Converts a [DatumError] to an [Error].
    ///
    /// _Added in 1.1.0._
    pub(crate) fn error_from_datum(e: DatumError) -> Error {
        serde::de::Error::custom(e)
    }
}

pub mod de;
pub mod ser;

/// Document layout descriptor.
///
/// Added in 1.2.0.
#[non_exhaustive]
#[derive(Clone, PartialEq, Eq)]
pub enum DocLayout {
    Plain,
    Root,
}

impl DocLayout {
    /// Deserialize from a token iterator.
    pub fn deserialize_tokens<'a, V: Deserialize<'a>, B: Default + Deref<Target = str>>(
        &self,
        iterator: &mut dyn Iterator<Item = DatumResult<DatumToken<B>>>,
    ) -> error::Result<V> {
        match self {
            Self::Plain => {
                let mut it = de::PlainDeserializer::from_iterator(iterator);
                V::deserialize(&mut it)
            }
            Self::Root => {
                let mut it = de::RootDeserializer::from_iterator(iterator);
                V::deserialize(&mut it)
            }
        }
    }

    /// Serialize to a [Write] implementation (including [String]).
    ///
    /// _Importantly, this should not be used in a 'chained' fashion, as the writing state is reset between calls._
    /// _This is for one-off writes only._
    pub fn serialize_to<V: Serialize>(
        &self,
        v: &V,
        w: &mut dyn Write,
        style: ser::Style,
    ) -> error::Result<()> {
        match self {
            Self::Plain => {
                let mut it = ser::PlainSerializer::new(w, style);
                v.serialize(&mut it)
            }
            Self::Root => {
                let mut it = ser::RootSerializer::new(w, style);
                v.serialize(&mut it)
            }
        }
    }

    /// Deserialize from a str.
    #[cfg(feature = "alloc")]
    pub fn deserialize_str<'a, V: Deserialize<'a>, S: Deref<Target = str>>(
        &self,
        text: S,
    ) -> error::Result<V> {
        use crate::{datum_char_to_token_pipeline, IntoViaDatumPipe};

        let mut token_iterator = text.chars().via_datum_pipe(datum_char_to_token_pipeline());
        self.deserialize_tokens(&mut token_iterator)
    }

    /// Serialize to a [alloc::string::String].
    #[cfg(feature = "alloc")]
    pub fn serialize_to_string<V: Serialize>(
        &self,
        v: &V,
        style: ser::Style,
    ) -> error::Result<alloc::string::String> {
        let mut res = alloc::string::String::new();
        self.serialize_to(v, &mut res, style)?;
        Ok(res)
    }

    /// Deserialize from a file. _Beware: Allocates room for the whole file. Completely ignores trailing values._
    #[cfg(feature = "std")]
    pub fn deserialize_file<'a, V: Deserialize<'a>, P: AsRef<std::path::Path>>(
        &self,
        path: P,
    ) -> error::Result<V> {
        use serde::de::Error;
        let file = std::fs::read_to_string(path).map_err(|e| error::Error::custom(e))?;
        self.deserialize_str(file)
    }
}

#[cfg(feature = "alloc")]
#[cfg(feature = "_serde_test_features")]
#[cfg(test)]
mod tests;
