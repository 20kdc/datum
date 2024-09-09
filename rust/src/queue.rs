/*
 * datum-rs - Quick to implement S-expression format
 * Written starting in 2024 by contributors (see CREDITS.txt at repository's root)
 * To the extent possible under law, the author(s) have dedicated all copyright and related and neighboring rights to this software to the public domain worldwide. This software is distributed without any warranty.
 * A copy of the Unlicense should have been supplied as COPYING.txt in this repository. Alternatively, you can find it at <https://unlicense.org/>.
 */

use crate::unary;

/// Type to convert a Self [unary::Num] (length) into a queue of the given type.
///
/// _Added in 1.2.0._
pub trait DatumUnaryNumIntoQueue<T>: unary::Num {
    type Queue: DatumBoundedQueue<T>;
}

impl<T> DatumUnaryNumIntoQueue<T> for unary::C0 {
    type Queue = ();
}

impl<T, Rem: DatumUnaryNumIntoQueue<T>> DatumUnaryNumIntoQueue<T> for unary::Digit<Rem> {
    type Queue = Option<(T, Rem::Queue)>;
}

/// Fixed-size queue-like apparatus, implemented for () and Option(T, Q).
///
/// _Added in 1.2.0._
pub trait DatumBoundedQueue<T>: Sized + Default {
    type Len: unary::Num;
    fn push_back(&mut self, v: T);
    fn pop_front(&mut self) -> Option<T>;
}

impl<T> DatumBoundedQueue<T> for () {
    type Len = unary::C0;

    fn push_back(&mut self, _v: T) {
        panic!("Ran out of DatumBoundedQueue space")
    }
    fn pop_front(&mut self) -> Option<T> {
        None
    }
}

impl<T, Q: DatumBoundedQueue<T>> DatumBoundedQueue<T> for Option<(T, Q)> {
    type Len = unary::Digit<Q::Len>;

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
