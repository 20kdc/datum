/*
 * datum-rs - Quick to implement S-expression format
 * Written starting in 2024 by contributors (see CREDITS.txt at repository's root)
 * To the extent possible under law, the author(s) have dedicated all copyright and related and neighboring rights to this software to the public domain worldwide. This software is distributed without any warranty.
 * A copy of the Unlicense should have been supplied as COPYING.txt in this repository. Alternatively, you can find it at <https://unlicense.org/>.
 */

use crate::{
    DatumBoundedPipe, DatumBoundedQueue, DatumComposePipe, DatumError, DatumOffset, DatumPipe,
    DatumResult,
};

/// [DatumPipe] with an internal buffer.
///
/// Switching between APIs without clearing the buffer may produce odd results (discarding buffer content).
///
/// buffer_feed and buffer_next provide an API using the internal buffer, rather than callbacks.
///
/// _Added in 1.2.0._
pub trait DatumBufferedPipe {
    type Input;
    type Output;
    /// Clears the buffer and sets its contents from the results of the given input.
    fn buffer_feed(&mut self, at: DatumOffset, i: Option<Self::Input>);
    /// Forcibly clears the buffer.
    fn buffer_clear(&mut self);
    /// Gets the next value in the buffer.
    fn buffer_next(&mut self) -> Option<DatumResult<(DatumOffset, Self::Output)>>;
}

/// Converts into a [DatumBufferedPipe] implementation.
/// This implies the type gains whatever internal buffers are necessary.
///
/// _Added in 1.2.0._
pub trait IntoDatumBufferedPipe: DatumPipe {
    type IntoBufferedPipe: DatumBufferedPipe<Input = Self::Input, Output = Self::Output> + Sized;
    fn into_buf_pipe(self) -> Self::IntoBufferedPipe;
}

impl<V: DatumBoundedPipe> IntoDatumBufferedPipe for V {
    type IntoBufferedPipe = DatumBufferedBoundedPipe<V>;
    fn into_buf_pipe(self) -> Self::IntoBufferedPipe {
        DatumBufferedBoundedPipe::new(self)
    }
}

/// Wraps a [DatumBoundedPipe] to make it a [DatumBufferedPipe].
///
/// _Added in 1.2.0._
#[derive(Clone, Copy, Debug)]
pub struct DatumBufferedBoundedPipe<P: DatumBoundedPipe>(P, P::OutputQueue, Option<DatumError>);

impl<P: DatumBoundedPipe> DatumBufferedBoundedPipe<P> {
    pub fn new(p: P) -> Self {
        DatumBufferedBoundedPipe(p, Default::default(), None)
    }
}

impl<P: DatumBoundedPipe + Default> Default for DatumBufferedBoundedPipe<P> {
    fn default() -> Self {
        Self(Default::default(), Default::default(), None)
    }
}

impl<P: DatumBoundedPipe> DatumBufferedPipe for DatumBufferedBoundedPipe<P> {
    type Input = P::Input;
    type Output = P::Output;
    fn buffer_feed(&mut self, at: DatumOffset, i: Option<Self::Input>) {
        let mut queue: P::OutputQueue = Default::default();
        self.2 = self
            .0
            .feed(at, i, &mut |ofs, v| {
                queue.push_back((ofs, v));
                Ok(())
            })
            .err();
        self.1 = queue;
    }
    fn buffer_clear(&mut self) {
        self.1 = Default::default();
        self.2 = None;
    }
    fn buffer_next(&mut self) -> Option<DatumResult<(DatumOffset, Self::Output)>> {
        if let Some(res) = self.1.pop_front().map(|v| Ok(v)) {
            Some(res)
        } else {
            self.2.take().map(|v| Err(v))
        }
    }
}

impl<A: IntoDatumBufferedPipe, B: IntoDatumBufferedPipe<Input = A::Output>> IntoDatumBufferedPipe
    for DatumComposePipe<A, B>
{
    type IntoBufferedPipe = DatumBufferedComposePipe<A::IntoBufferedPipe, B::IntoBufferedPipe>;
    fn into_buf_pipe(self) -> Self::IntoBufferedPipe {
        DatumBufferedComposePipe::new(self.0.into_buf_pipe(), self.1.into_buf_pipe())
    }
}

/// Composed buffered pipe.
///
/// _Added in 1.2.0._
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct DatumBufferedComposePipe<A: DatumBufferedPipe, B: DatumBufferedPipe<Input = A::Output>> {
    a: A,
    b: B,
    offset: DatumOffset,
    waiting_eof: bool,
}

impl<A: DatumBufferedPipe, B: DatumBufferedPipe<Input = A::Output>> DatumBufferedComposePipe<A, B> {
    pub fn new(a: A, b: B) -> Self {
        Self {
            a,
            b,
            offset: 0,
            waiting_eof: false,
        }
    }
}

impl<A: DatumBufferedPipe + Default, B: DatumBufferedPipe<Input = A::Output> + Default> Default
    for DatumBufferedComposePipe<A, B>
{
    fn default() -> Self {
        DatumBufferedComposePipe::new(A::default(), B::default())
    }
}

impl<A: DatumBufferedPipe, B: DatumBufferedPipe<Input = A::Output>> DatumBufferedPipe
    for DatumBufferedComposePipe<A, B>
{
    type Input = A::Input;
    type Output = B::Output;

    fn buffer_feed(&mut self, at: DatumOffset, i: Option<Self::Input>) {
        self.offset = at;
        self.b.buffer_clear();
        self.waiting_eof |= i.is_none();
        self.a.buffer_feed(at, i);
    }
    fn buffer_clear(&mut self) {
        self.a.buffer_clear();
        self.b.buffer_clear();
        self.waiting_eof = false;
    }
    fn buffer_next(&mut self) -> Option<DatumResult<(DatumOffset, Self::Output)>> {
        loop {
            // B buffer check
            if let Some(v) = self.b.buffer_next() {
                return Some(v);
            }
            // A buffer check
            match self.a.buffer_next() {
                Some(Ok(v)) => {
                    self.b.buffer_feed(v.0, Some(v.1));
                    continue;
                }
                Some(Err(err)) => return Some(Err(err)),
                None => {}
            }
            // we fed an EOF to A earlier, we have to now inform B
            if self.waiting_eof {
                self.waiting_eof = false;
                self.b.buffer_feed(self.offset, None);
                continue;
            }
            // done
            return None;
        }
    }
}
