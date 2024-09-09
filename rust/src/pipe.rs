/*
 * datum-rs - Quick to implement S-expression format
 * Written starting in 2024 by contributors (see CREDITS.txt at repository's root)
 * To the extent possible under law, the author(s) have dedicated all copyright and related and neighboring rights to this software to the public domain worldwide. This software is distributed without any warranty.
 * A copy of the Unlicense should have been supplied as COPYING.txt in this repository. Alternatively, you can find it at <https://unlicense.org/>.
 */

use crate::{DatumBoundedQueue, DatumBoundedQueueMul, DatumBoundedQueueOfType, DatumOffset, DatumResult};

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

/// Bounded queue of length 1.
///
/// _Added in 1.2.0._
pub type DatumBoundedQueue1<V> = Option<((DatumOffset, V), ())>;

/// Bounded queue of length 2.
///
/// _Added in 1.2.0._
pub type DatumBoundedQueue2<V> = Option<((DatumOffset, V), DatumBoundedQueue1<V>)>;

/// [DatumPipe] of bounded output size.
/// Notably, this can never apply to [DatumComposePipe].
/// However, it's still useful as a building block.
///
/// _Added in 1.2.0._
pub trait DatumBoundedPipe: DatumPipe {
    /// Output queue type. Allows writing no-alloc code which can buffer the output of a DatumBoundedPipe.
    type OutputQueue: DatumBoundedQueue<(DatumOffset, Self::Output)>;
}

/// Composed pipe.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct DatumComposePipe<A: DatumPipe, B: DatumPipe<Input = A::Output>>(pub A, pub B);

// Usually pipelines are entirely type-described, so implement Default where possible.
// This allows the idiom `let x: PipelineType = PipelineType::default();`.
// _Added in 1.1.0._
impl<A: DatumPipe + Default, B: DatumPipe<Input = A::Output> + Default> Default
    for DatumComposePipe<A, B>
{
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

// In a composed buffer pair, in response to one feed call, A can output len(A).
// B can output (len(A) + 1) * len(B).

impl<
        A: DatumBoundedPipe<OutputQueue = AQ>,
        B: DatumBoundedPipe<Input = A::Output, OutputQueue = BQ>,
        AQ: DatumBoundedQueueOfType<(DatumOffset, B::Output), Changed = AQC>,
        AQC: DatumBoundedQueue<(DatumOffset, B::Output), Inc = AQCI>,
        AQCI: DatumBoundedQueue<(DatumOffset, B::Output)>,
        BQ: DatumBoundedQueueMul<(DatumOffset, B::Output), AQCI>
    > DatumBoundedPipe for DatumComposePipe<A, B>
{
    type OutputQueue = BQ::Mul;
}
