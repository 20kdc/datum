/*
 * datum-rs - Quick to implement S-expression format
 * Written starting in 2024 by contributors (see CREDITS.txt at repository's root)
 * To the extent possible under law, the author(s) have dedicated all copyright and related and neighboring rights to this software to the public domain worldwide. This software is distributed without any warranty.
 * A copy of the Unlicense should have been supplied as COPYING.txt in this repository. Alternatively, you can find it at <https://unlicense.org/>.
 */

use crate::{
    sealant::DatumAllPurposeTraitSealant, DatumBoundedPipe, DatumBoundedQueue, DatumError,
    DatumOffset, DatumPipe, DatumResult,
};

/// [DatumPipe] with an internal buffer.
///
/// Switching between APIs without clearing the buffer may produce odd results (discarding buffer content).
///
/// buffer_feed and buffer_next provide an API using the internal buffer, rather than callbacks.
///
/// _This trait is sealed and cannot be implemented by external code._ (This is to ensure that `DatumBufPipe: IntoDatumBufPipe`)
///
/// The expected implementations are:
///
/// * [DatumBufBoundedPipe]
/// * [DatumBufComposePipe]
///
/// _Added in 1.2.0._
pub trait DatumBufPipe {
    type Input;
    type Output;
    /// Clears the buffer and sets its contents from the results of the given input.
    fn buffer_feed(&mut self, at: DatumOffset, i: Option<Self::Input>);
    /// Forcibly clears the buffer.
    fn buffer_clear(&mut self);
    /// Gets the next value in the buffer.
    fn buffer_next(&mut self) -> Option<DatumResult<(DatumOffset, Self::Output)>>;

    /// This trait is sealed and not to be implemented in downstream crates.
    fn __sealed(self) -> DatumAllPurposeTraitSealant<Self>;
}

// Implementing DatumPipe for DatumBufPipe allows IntoDatumBufPipe to be defined on DatumBufPipe.
impl<V: DatumBufPipe> DatumPipe for V {
    type Input = V::Input;
    type Output = V::Output;

    fn feed<F: FnMut(DatumOffset, Self::Output) -> DatumResult<()>>(
        &mut self,
        at: DatumOffset,
        i: Option<Self::Input>,
        f: &mut F,
    ) -> DatumResult<()> {
        self.buffer_feed(at, i);
        loop {
            match self.buffer_next() {
                None => break,
                Some(val) => {
                    let val = val?;
                    f(val.0, val.1)?;
                }
            }
        }
        Ok(())
    }
}

/// Converts into a [DatumBufPipe] implementation.
/// This implies the type gains whatever internal buffers are necessary.
///
/// _Added in 1.2.0._
pub trait IntoDatumBufPipe: DatumPipe {
    type IntoBufferedPipe: DatumBufPipe<Input = Self::Input, Output = Self::Output> + Sized;
    fn into_buf_pipe(self) -> Self::IntoBufferedPipe;
}

impl<V: DatumBoundedPipe> IntoDatumBufPipe for V {
    type IntoBufferedPipe = DatumBufBoundedPipe<V>;
    fn into_buf_pipe(self) -> Self::IntoBufferedPipe {
        DatumBufBoundedPipe::new(self)
    }
}

/// Wraps a [DatumBoundedPipe] to make it a [DatumBufPipe].
///
/// _Added in 1.2.0._
#[derive(Clone, Copy, Debug)]
pub struct DatumBufBoundedPipe<P: DatumBoundedPipe>(P, P::OutputQueue, Option<DatumError>);

impl<P: DatumBoundedPipe> DatumBufBoundedPipe<P> {
    pub fn new(p: P) -> Self {
        DatumBufBoundedPipe(p, Default::default(), None)
    }
}

impl<P: DatumBoundedPipe> IntoDatumBufPipe for DatumBufBoundedPipe<P> {
    type IntoBufferedPipe = Self;
    fn into_buf_pipe(self) -> Self::IntoBufferedPipe {
        self
    }
}

impl<P: DatumBoundedPipe + Default> Default for DatumBufBoundedPipe<P> {
    fn default() -> Self {
        Self(Default::default(), Default::default(), None)
    }
}

impl<P: DatumBoundedPipe> DatumBufPipe for DatumBufBoundedPipe<P> {
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
        if let Some(res) = self.1.pop_front().map(Ok) {
            Some(res)
        } else {
            self.2.take().map(Err)
        }
    }
    fn __sealed(self) -> DatumAllPurposeTraitSealant<Self> {
        DatumAllPurposeTraitSealant::new()
    }
}
