/*
 * datum-rs - Quick to implement S-expression format
 * Written starting in 2024 by contributors (see CREDITS.txt at repository's root)
 * To the extent possible under law, the author(s) have dedicated all copyright and related and neighboring rights to this software to the public domain worldwide. This software is distributed without any warranty.
 * A copy of the Unlicense should have been supplied as COPYING.txt in this repository. Alternatively, you can find it at <https://unlicense.org/>.
 */

//! Serde serializer/deserializer implementation and supporting types.
//!
//! _Added in 1.1.0._

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

#[cfg(feature = "alloc")]
#[cfg(feature = "_serde_test_features")]
#[cfg(test)]
mod de_tests;

#[cfg(feature = "alloc")]
#[cfg(feature = "_serde_test_features")]
#[cfg(test)]
mod ser_tests;
