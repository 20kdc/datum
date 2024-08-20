/*
 * datum-rs - Quick to implement S-expression format
 * Written starting in 2024 by contributors (see CREDITS.txt at repository's root)
 * To the extent possible under law, the author(s) have dedicated all copyright and related and neighboring rights to this software to the public domain worldwide. This software is distributed without any warranty.
 * A copy of the Unlicense should have been supplied as COPYING.txt in this repository. Alternatively, you can find it at <https://unlicense.org/>.
 */

use core::{future::Future, pin::pin, ptr::null, task::{Context, Poll, RawWaker, RawWakerVTable, Waker}};

#[inline]
fn raw_waker_new(_: *const ()) -> RawWaker {
    RawWaker::new(null(), &RawWakerVTable::new(raw_waker_new, raw_waker_nop, raw_waker_nop, raw_waker_nop))
}
#[inline]
fn raw_waker_nop(_: *const ()) {
}

#[inline]
fn unwakable() -> Waker {
    unsafe { Waker::from_raw(raw_waker_new(null())) }
}

pub trait AsyncUnwrap<V>: Future<Output = V> + Sized {
    /// Polls the future. If it does not immediately succeed, panics.
    fn unwrap(self) -> V {
        self.expect("called AsyncUnwrap::unwrap, should have received value")
    }
    /// Polls the future. If it does not immediately succeed, panics with `msg`.
    fn expect(self, msg: &str) -> V {
        let waker = unwakable();
        let mut ctx = Context::from_waker(&waker);
        let mut pinned = pin!(self);
        if let Poll::Ready(res) = pinned.as_mut().poll(&mut ctx) {
            return res;
        } else {
            panic!("{}", msg)
        }
    }
}

impl<X: Future + Sized> AsyncUnwrap<X::Output> for X {}
