/*
 * datum-rs - Quick to implement S-expression format
 * Written starting in 2024 by contributors (see CREDITS.txt at repository's root)
 * To the extent possible under law, the author(s) have dedicated all copyright and related and neighboring rights to this software to the public domain worldwide. This software is distributed without any warranty.
 * A copy of the Unlicense should have been supplied as COPYING.txt in this repository. Alternatively, you can find it at <https://unlicense.org/>.
 */

//! Unary compile-time maths for use by queues and such.

/// Trait register copier assistant
///
/// _Added in 1.2.0._
pub trait TyCo {
    type R1;
    type R2;
    type R3;
    type R4;
}
impl<V> TyCo for V {
    type R1 = V;
    type R2 = V;
    type R3 = V;
    type R4 = V;
}

/// Compile-time unary type-system value. Either [C0] or [Digit].
///
/// _Added in 1.2.0._
pub trait Num {
    /// Self + 1
    type Inc: Num;
}

/// Unary zero.
///
/// _Added in 1.2.0._
pub enum C0 {}

/// Unary one.
///
/// _Added in 1.2.0._
pub struct Digit<V: Num>(V);

impl Num for C0 {
    type Inc = Digit<Self>;
}

impl<V: Num> Num for Digit<V> {
    type Inc = Digit<Self>;
}

/// Unary: Binary operator results
///
/// _Added in 1.2.0._
pub trait Add<O: Num> {
    /// Added together.
    type Add: Num;
}

/// Unary: Multiplier
///
/// _Added in 1.2.0._
pub trait Mul<O: Num> {
    /// Multiplied
    type Mul: Num;
}

impl<O: Num> Add<O> for C0 {
    type Add = O;
}

impl<
        SelfDec: Num + TyCo<R1 = OpSelfDecOtherInc>,
        Other: Num<Inc = OtherInc>,
        OtherInc: Num,
        // ---
        OpSelfDecOtherInc: Add<OtherInc>,
    > Add<Other> for Digit<SelfDec>
{
    // SelfDec = Self - 1
    // OtherInc = Other + 1
    // Add = SelfDec + OtherInc
    // Therefore Add = Self + O
    type Add = OpSelfDecOtherInc::Add;
}

impl<O: Num> Mul<O> for C0 {
    type Mul = C0;
}

impl<
        SelfDec: Num + TyCo<R1 = OpSelfDecOther>,
        Other: Num<Inc = OtherInc>,
        OtherInc: Num,
        // ---
        OpSelfDecOther: Mul<Other, Mul = OMQ>,
        OpOMQOther: Add<Other>,
        // ---
        OMQ: Num + TyCo<R1 = OpOMQOther>,
    > Mul<Other> for Digit<SelfDec>
{
    // SelfDec = Self - 1
    // OMQ = SelfDec * Other
    // Add = OMQ + Other
    // Therefore Add = Self * O
    type Mul = OpOMQOther::Add;
}

/// Unary: Constant
///
/// _Added in 1.2.0._
pub type C1 = Digit<C0>;

/// Unary: Constant
///
/// _Added in 1.2.0._
pub type C2 = Digit<C1>;

/// Unary: Constant
///
/// _Added in 1.2.0._
pub type C3 = Digit<C2>;

/// Unary: Constant
///
/// _Added in 1.2.0._
pub type C4 = Digit<C3>;
