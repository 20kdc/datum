/*
 * datum-rs - Quick to implement S-expression format
 * Written starting in 2024 by contributors (see CREDITS.txt at repository's root)
 * To the extent possible under law, the author(s) have dedicated all copyright and related and neighboring rights to this software to the public domain worldwide. This software is distributed without any warranty.
 * A copy of the Unlicense should have been supplied as COPYING.txt in this repository. Alternatively, you can find it at <https://unlicense.org/>.
 */

/// Trait register copier assistant.
/// Basically this is just a bunch of dummy slots that a trait can assign to its other generics.
/// If you need more than 4 of these, copy into a register which you just use for more copying.
///
/// _Added in 1.2.0._
pub trait DatumTypeCopy {
    type R1;
    type R2;
    type R3;
    type R4;
}
impl<V> DatumTypeCopy for V {
    type R1 = V;
    type R2 = V;
    type R3 = V;
    type R4 = V;
}

/// Fixed-size queue-like apparatus, implemented for () and Option(T, Q).
///
/// _Added in 1.2.0._
pub trait DatumBoundedQueue<T>: Sized + Default {
    /// A queue of +1 length.
    type Inc: DatumBoundedQueue<T>;

    fn push_back(&mut self, v: T);
    fn pop_front(&mut self) -> Option<T>;
}

/// Fixed-size queue-like apparatus: Binary operator results
///
/// _Added in 1.2.0._
pub trait DatumBoundedQueueAdd<T, O: DatumBoundedQueue<T>> {
    /// Added together.
    type Add: DatumBoundedQueue<T>;
}

/// Fixed-size queue-like apparatus: Multiplier
///
/// _Added in 1.2.0._
pub trait DatumBoundedQueueMul<T, O: DatumBoundedQueue<T>> {
    /// Multiplied
    type Mul: DatumBoundedQueue<T>;
}

/// Fixed-size queue-like apparatus: type changer
///
/// _Added in 1.2.0._
pub trait DatumBoundedQueueOfType<T> {
    /// Adjusted type.
    type Changed: DatumBoundedQueue<T>;
}

impl<T> DatumBoundedQueue<T> for () {
    type Inc = Option<(T, Self)>;

    fn push_back(&mut self, _v: T) {
        panic!("Ran out of DatumBoundedQueue space")
    }
    fn pop_front(&mut self) -> Option<T> {
        None
    }
}

impl<T> DatumBoundedQueueOfType<T> for () {
    type Changed = ();
}

impl<T, Q: DatumBoundedQueue<T>> DatumBoundedQueue<T> for Option<(T, Q)> {
    type Inc = Option<(T, Self)>;

    fn push_back(&mut self, v: T) {
        if let Some(q) = self {
            q.1.push_back(v);
        } else {
            *self = Some((v, Default::default()));
        }
    }
    fn pop_front(&mut self) -> Option<T> {
        let tmp = self.take();
        match tmp {
            Self::None => None,
            Self::Some((v, mut q)) => {
                loop {
                    match q.pop_front() {
                        None => break,
                        Some(v2) => self.push_back(v2),
                    }
                }
                Some(v)
            }
        }
    }
}

impl<T, Q: DatumBoundedQueueOfType<N>, N> DatumBoundedQueueOfType<N> for Option<(T, Q)> {
    type Changed = Option<(N, Q::Changed)>;
}

impl<T, O: DatumBoundedQueue<T>> DatumBoundedQueueAdd<T, O> for () {
    type Add = O;
}

impl<
        T,
        SelfDec: DatumBoundedQueue<T> + DatumTypeCopy<R1 = OpSelfDecOtherInc>,
        Other: DatumBoundedQueue<T, Inc = OtherInc>,
        OtherInc: DatumBoundedQueue<T>,
        // ---
        OpSelfDecOtherInc: DatumBoundedQueueAdd<T, OtherInc>,
    > DatumBoundedQueueAdd<T, Other> for Option<(T, SelfDec)>
{
    // SelfDec = Self - 1
    // OtherInc = Other + 1
    // Add = SelfDec + OtherInc
    // Therefore Add = Self + O
    type Add = OpSelfDecOtherInc::Add;
}

impl<T, O: DatumBoundedQueue<T>> DatumBoundedQueueMul<T, O> for () {
    type Mul = ();
}

impl<
        T,
        SelfDec: DatumBoundedQueue<T> + DatumTypeCopy<R1 = OpSelfDecOther>,
        Other: DatumBoundedQueue<T, Inc = OtherInc>,
        OtherInc: DatumBoundedQueue<T>,
        // ---
        OpSelfDecOther: DatumBoundedQueueMul<T, Other, Mul = OMQ>,
        OpOMQOther: DatumBoundedQueueAdd<T, Other>,
        // ---
        OMQ: DatumBoundedQueue<T> + DatumTypeCopy<R1 = OpOMQOther>
    > DatumBoundedQueueMul<T, Other> for Option<(T, SelfDec)>
{
    // SelfDec = Self - 1
    // OMQ = SelfDec * Other
    // Add = OMQ + Other
    // Therefore Add = Self * O
    type Mul = OpOMQOther::Add;
}
