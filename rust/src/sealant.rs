/*
 * datum-rs - Quick to implement S-expression format
 * Written starting in 2024 by contributors (see CREDITS.txt at repository's root)
 * To the extent possible under law, the author(s) have dedicated all copyright and related and neighboring rights to this software to the public domain worldwide. This software is distributed without any warranty.
 * A copy of the Unlicense should have been supplied as COPYING.txt in this repository. Alternatively, you can find it at <https://unlicense.org/>.
 */

use core::marker::PhantomData;

/// This type is used in traits which are _not to be implemented by downstream crates._
///
/// `fn __sealed(self) -> $crate::DatumAllPurposeTraitSealant<Self>;`
///
/// _Added in 1.2.0._
pub struct DatumAllPurposeTraitSealant<T: ?Sized>(PhantomData<T>);
impl<T> DatumAllPurposeTraitSealant<T> {
    /// Creates a new instance of the sealant.
    /// An arbitrary primitive to create [DatumAllPurposeTraitSealant] for a given type must not leak outside of the crate.
    pub(crate) fn new() -> DatumAllPurposeTraitSealant<T> {
        DatumAllPurposeTraitSealant(PhantomData)
    }
}
