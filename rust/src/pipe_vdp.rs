/*
 * datum-rs - Quick to implement S-expression format
 * Written starting in 2024 by contributors (see CREDITS.txt at repository's root)
 * To the extent possible under law, the author(s) have dedicated all copyright and related and neighboring rights to this software to the public domain worldwide. This software is distributed without any warranty.
 * A copy of the Unlicense should have been supplied as COPYING.txt in this repository. Alternatively, you can find it at <https://unlicense.org/>.
 */

#[cfg(feature = "alloc")]
mod vdp {
    use alloc::collections::VecDeque;

    use crate::{DatumOffset, DatumPipe, DatumResult};

    /// This is used in [IntoViaDatumPipe::via_datum_pipe].
    #[derive(Clone)]
    pub struct ViaDatumPipe<I: Iterator<Item = S>, S, P: DatumPipe<Input = S>> {
        offset: DatumOffset,
        iterator: I,
        pipeline: P,
        buffer: VecDeque<P::Output>,
        eof: bool,
    }

    impl<I: Iterator<Item = S>, S, P: DatumPipe<Input = S>> Iterator for ViaDatumPipe<I, S, P> {
        type Item = DatumResult<P::Output>;

        fn next(&mut self) -> Option<Self::Item> {
            loop {
                if let Some(res) = self.buffer.pop_front() {
                    return Some(Ok(res));
                } else if self.eof {
                    return None;
                } else {
                    let base_res = self.iterator.next();
                    let buffer = &mut self.buffer;
                    if base_res.is_none() {
                        self.eof = true;
                    }
                    let res = self.pipeline.feed(self.offset, base_res, &mut |_, v| {
                        buffer.push_back(v);
                        Ok(())
                    });
                    if let Err(err) = res {
                        return Some(Err(err));
                    }
                    self.offset += 1;
                }
            }
        }
    }

    /// This is used to provide [IntoViaDatumPipe::via_datum_pipe] on [Iterator].
    pub trait IntoViaDatumPipe<I>: Iterator<Item = I> + Sized {
        /// Parses/handles elements via a [DatumPipe].
        /// The resulting [ViaDatumPipe] maintains an internal [VecDeque] buffer of values to return.
        /// When the iterator runs out of elements, an EOF will be signalled.
        /// At that point, the pipe iterator will no longer retrieve elements from the source.
        /// Offsets are internally managed and start at 0.
        fn via_datum_pipe<P: DatumPipe<Input = I>>(self, pipe: P) -> ViaDatumPipe<Self, I, P>;
    }

    impl<I, V: Iterator<Item = I> + Sized> IntoViaDatumPipe<I> for V {
        fn via_datum_pipe<P: DatumPipe<Input = I>>(self, pipe: P) -> ViaDatumPipe<Self, I, P> {
            ViaDatumPipe {
                offset: 0,
                iterator: self,
                pipeline: pipe,
                buffer: VecDeque::new(),
                eof: false,
            }
        }
    }
}

#[cfg(feature = "alloc")]
pub use vdp::*;

mod vdbp {
    use crate::{DatumBufPipe, DatumOffset, DatumResult, IntoDatumBufPipe};

    /// This is used in [IntoViaDatumBufPipe::via_datum_buf_pipe].
    ///
    /// _Added in 1.2.0._
    #[derive(Clone)]
    pub struct ViaDatumBufPipe<I: Iterator<Item = S>, S, P: DatumBufPipe<Input = S>> {
        offset: DatumOffset,
        iterator: I,
        pipeline: P,
        eof: bool,
    }

    impl<I: Iterator<Item = S>, S, P: DatumBufPipe<Input = S>> Iterator for ViaDatumBufPipe<I, S, P> {
        type Item = DatumResult<P::Output>;

        fn next(&mut self) -> Option<Self::Item> {
            loop {
                let res = self.pipeline.buffer_next();
                if let Some(res) = res.map(|v| v.map(|v| v.1)) {
                    return Some(res);
                } else if self.eof {
                    return None;
                } else {
                    // ran out of buffer, try iterator
                    let val = self.iterator.next();
                    self.eof |= val.is_none();
                    self.pipeline.buffer_feed(self.offset, val);
                    self.offset += 1;
                }
            }
        }
    }

    /// This is used to provide [IntoViaDatumBufPipe::via_datum_buf_pipe] on [Iterator].
    ///
    /// _Added in 1.2.0._
    pub trait IntoViaDatumBufPipe<I>: Iterator<Item = I> + Sized {
        /// Parses/handles elements via a [DatumBufPipe] (or convertible via [IntoDatumBufPipe]).
        /// The resulting [ViaDatumBufPipe] maintains an internal buffer of values to return.
        /// When the iterator runs out of elements, an EOF will be signalled.
        /// At that point, the pipe iterator will no longer retrieve elements from the source.
        /// Offsets are internally managed and start at 0.
        fn via_datum_buf_pipe<P: IntoDatumBufPipe<Input = I>>(
            self,
            pipe: P,
        ) -> ViaDatumBufPipe<Self, I, P::IntoBufferedPipe>;
    }

    impl<I, V: Iterator<Item = I> + Sized> IntoViaDatumBufPipe<I> for V {
        fn via_datum_buf_pipe<P: IntoDatumBufPipe<Input = I>>(
            self,
            pipe: P,
        ) -> ViaDatumBufPipe<Self, I, P::IntoBufferedPipe> {
            ViaDatumBufPipe {
                offset: 0,
                iterator: self,
                pipeline: pipe.into_buf_pipe(),
                eof: false,
            }
        }
    }
}

pub use vdbp::*;
