/*
 * datum-rs - Quick to implement S-expression format
 * Written starting in 2024 by contributors (see CREDITS.txt at repository's root)
 * To the extent possible under law, the author(s) have dedicated all copyright and related and neighboring rights to this software to the public domain worldwide. This software is distributed without any warranty.
 * A copy of the Unlicense should have been supplied as COPYING.txt in this repository. Alternatively, you can find it at <https://unlicense.org/>.
 */

use core::fmt::Display;
use std::{cell::Cell, marker::PhantomData};

#[cfg(feature = "alloc")]
use alloc::collections::VecDeque;

/// Any error producible by Datum.
#[non_exhaustive]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DatumErrorKind {
    /// Ran out of room in a buffer.
    /// Happens in [crate::DatumPipeTokenizer] if the passed [core::fmt::Write] implementor fails.
    OutOfRoom,
    /// Interrupted; you should re-parse with more data.
    /// Happens on various EOF conditions.
    Interrupted,
    /// Bad data.
    BadData,
    /// Custom error signal. Will never be generated by Datum.
    /// Beware that external libraries may have their own user stages.
    Custom,
}

impl Display for DatumErrorKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Diagnostic offset. Units are dependent on how you use it.
pub type DatumOffset = u64;

/// Datum error.
/// Notably, writing uses a different error type, so these are read-focused.
#[derive(Clone, Copy, Debug)]
pub struct DatumError {
    /// Kind of error. Useful to distingulish EOF errors from non-EOF errors.
    pub kind: DatumErrorKind,
    /// Error occurred at this offset. Offsets are entirely caller-supplied, except when iterator wrapping is in use.
    pub offset: DatumOffset,
    /// Message.
    pub message: &'static str,
}

// this should be migrated to core::error::Error once that stabilizes

#[cfg(feature = "std")]
impl std::error::Error for DatumError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
    fn description(&self) -> &str {
        self.message
    }
    fn cause(&self) -> Option<&dyn std::error::Error> {
        None
    }
}

impl Display for DatumError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{} @ {}: {}", self.kind, self.offset, self.message)
    }
}

/// Creates a [DatumError].
/// Beware: If `detailed_errors` is off, messages are discarded.
#[cfg(feature = "detailed_errors")]
#[macro_export]
macro_rules! datum_error {
    ($kind:ident, $offset:expr, $info:literal) => {
        $crate::DatumError {
            kind: $crate::DatumErrorKind::$kind,
            offset: $offset,
            message: $info,
        }
    };
}

/// Creates a [DatumError].
/// Beware: If `detailed_errors` is off, messages are discarded.
#[cfg(not(feature = "detailed_errors"))]
#[macro_export]
macro_rules! datum_error {
    ($kind:ident, $offset:expr, $info:literal) => {
        $crate::DatumError {
            kind: $crate::DatumErrorKind::$kind,
            offset: $offset,
            message: "",
        }
    };
}

/// Datum result for the given value.
pub type DatumResult<V> = Result<V, DatumError>;

/// Generic "input X, get Y" function
pub trait DatumPipe {
    type Input;
    type Output;

    /// Feeds in I, and you may some amount of O.
    /// If None is passed, EOF happened.
    /// Beware that EOF offsets will 'skip' the function for future stages, so transforming offsets here is not okay. The primary purpose of the offset output is to ensure offsets reliably point at the start of an output, rather than the end; this can be used for various useful purposes.
    /// Because of that, offsets may "jump backwards" if it's useful to do so (this happens in the tokenizer).
    fn feed<F: FnMut(DatumOffset, Self::Output) -> DatumResult<()>>(
        &mut self,
        at: DatumOffset,
        i: Option<Self::Input>,
        f: &mut F,
    ) -> DatumResult<()>;

    /// Feeds into a vec or similar from a slice.
    /// Offsets are automatically managed, starting from 0 and increasing by 1 for each input element.
    /// Can also automatically trigger EOF.
    /// ```
    /// use datum::{DatumDecoder, DatumPipe};
    /// let mut decoder = DatumDecoder::default();
    /// let mut results = vec![];
    /// decoder.feed_iter_to_vec(&mut results, "example text".chars(), true);
    /// assert_eq!(results.len(), 12);
    /// ```
    fn feed_iter_to_vec<S: IntoIterator<Item = Self::Input>, V: Extend<Self::Output>>(
        &mut self,
        target: &mut V,
        source: S,
        eof: bool,
    ) -> DatumResult<()> {
        let mut offset: DatumOffset = 0;
        for v in source {
            self.feed(offset, Some(v), &mut |_, v| {
                target.extend(Some(v));
                Ok(())
            })?;
            offset += 1;
        }
        if eof {
            self.feed(offset, None, &mut |_, v| {
                target.extend(Some(v));
                Ok(())
            })
        } else {
            Ok(())
        }
    }

    /// Composes with another pipeline.
    fn compose<P: DatumPipe<Input = Self::Output>>(self, other: P) -> DatumComposePipe<Self, P>
    where
        Self: Sized,
    {
        DatumComposePipe(self, other)
    }
}

/// Composed pipe.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct DatumComposePipe<A: DatumPipe, B: DatumPipe<Input = A::Output>>(pub A, pub B);

// Usually pipelines are entirely type-described, so implement Default where possible.
// This allows the idiom `let x: PipelineType = PipelineType::default();`.
// _Added in 1.1.0._
impl<A: DatumPipe + Default, B: DatumPipe<Input = A::Output> + Default> Default for DatumComposePipe<A, B> {
    fn default() -> Self {
        DatumComposePipe(A::default(), B::default())
    }
}

impl<A: DatumPipe, B: DatumPipe<Input = A::Output>> DatumPipe for DatumComposePipe<A, B> {
    type Input = A::Input;
    type Output = B::Output;

    fn feed<F: FnMut(DatumOffset, Self::Output) -> DatumResult<()>>(
        &mut self,
        at: DatumOffset,
        i: Option<Self::Input>,
        f: &mut F,
    ) -> DatumResult<()> {
        let m0 = &mut self.0;
        let m1 = &mut self.1;
        let was_none = i.is_none();
        m0.feed(at, i, &mut |o, v| m1.feed(o, Some(v), f))?;
        if was_none {
            m1.feed(at, None, f)
        } else {
            Ok(())
        }
    }
}

/// This is used in [IntoViaDatumPipe::via_datum_pipe].
#[cfg(feature = "alloc")]
#[derive(Clone)]
pub struct ViaDatumPipe<I: Iterator<Item = S>, S, P: DatumPipe<Input = S>> {
    offset: DatumOffset,
    iterator: I,
    pipeline: P,
    buffer: VecDeque<DatumResult<P::Output>>,
    eof: bool,
}

#[cfg(feature = "alloc")]
impl<I: Iterator<Item = S>, S, P: DatumPipe<Input = S>> Iterator for ViaDatumPipe<I, S, P> {
    type Item = DatumResult<P::Output>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(res) = self.buffer.pop_front() {
                return Some(res);
            } else if self.eof {
                return None;
            } else {
                let base_res = self.iterator.next();
                let buffer = &mut self.buffer;
                if base_res.is_none() {
                    self.eof = true;
                }
                if let Err(err) = self.pipeline.feed(self.offset, base_res, &mut |_, v| {
                    buffer.push_back(Ok(v));
                    Ok(())
                }) {
                    buffer.push_back(Err(err));
                }
                self.offset += 1;
            }
        }
    }
}

/// This is used to provide [IntoViaDatumPipe::via_datum_pipe] on [Iterator].
#[cfg(feature = "alloc")]
pub trait IntoViaDatumPipe<I>: Iterator<Item = I> + Sized {
    /// Parses/handles elements via a [DatumPipe].
    /// The resulting [ViaDatumPipe] maintains an internal [VecDeque] buffer of values to return.
    /// When the iterator runs out of elements, an EOF will be signalled.
    /// At that point, the pipe iterator will no longer retrieve elements from the source.
    /// Offsets are internally managed and start at 0.
    fn via_datum_pipe<P: DatumPipe<Input = I>>(self, pipe: P) -> ViaDatumPipe<Self, I, P>;
}

#[cfg(feature = "alloc")]
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

/// 'Tracking' pipeline stage, holding the line number in a cell by reference.
///
/// This stage increments a `Cell<u32>` each time a newline is encountered, after passing it to the callback.
///
/// Importantly, the line number tracker can be composed into and 'lost in' pipelines.
///
/// It can be inserted into byte or char pipelines, and is otherwise transparent.
///
/// This is very distinctly different to offsets, which are aimed at being able to, say, defer string parsing.
///
/// _Added in 1.1.0._
/// ```
/// use datum::{DatumPipe, DatumLineNumberTracker, DatumDecoder};
/// use core::cell::Cell;
/// let line_number = Cell::new(1);
/// let tracker: DatumLineNumberTracker<char> = DatumLineNumberTracker::new(&line_number);
/// // An arbitrarily complicated pipeline could go here.
/// let decoder = DatumDecoder::default();
/// let mut composed = tracker.compose(decoder);
/// composed.feed(0, Some('a'), &mut |_,_| Ok(())).unwrap();
/// composed.feed(0, Some('b'), &mut |_,_| Ok(())).unwrap();
/// composed.feed(0, Some('\n'), &mut |_,_| Ok(())).unwrap();
/// composed.feed(0, Some('\\'), &mut |_,_| Ok(())).unwrap();
/// // Oh no, we have an error (interrupted)...
/// composed.feed(0, None, &mut |_,_| Ok(())).unwrap_err();
/// // And we know it happened on line 2.
/// assert_eq!(line_number.get(), 2);
/// ```
pub struct DatumLineNumberTracker<'line_number, V: Copy + Into<u32>>(&'line_number Cell<u32>, PhantomData<V>);
impl<'line_number, V: Copy + Into<u32>> DatumLineNumberTracker<'line_number, V> {
    /// Creates a new DatumLineNumberTracker with the given line number storage.
    pub fn new(ln: &'line_number Cell<u32>) -> Self {
        Self(ln, PhantomData)
    }
}
impl<V: Copy + Into<u32>> DatumPipe for DatumLineNumberTracker<'_, V> {
    type Input = V;
    type Output = V;
    fn feed<F: FnMut(DatumOffset, Self::Output) -> DatumResult<()>>(
        &mut self,
        at: DatumOffset,
        i: Option<Self::Input>,
        f: &mut F,
    ) -> DatumResult<()> {
        if let Some(v) = i {
            let res = f(at, v);
            let chr: u32 = v.into();
            if chr == 10 {
                self.0.set(self.0.get().saturating_add(1));
            }
            res
        } else {
            Ok(())
        }
    }
}
