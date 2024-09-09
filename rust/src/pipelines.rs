/*
 * datum-rs - Quick to implement S-expression format
 * Written starting in 2024 by contributors (see CREDITS.txt at repository's root)
 * To the extent possible under law, the author(s) have dedicated all copyright and related and neighboring rights to this software to the public domain worldwide. This software is distributed without any warranty.
 * A copy of the Unlicense should have been supplied as COPYING.txt in this repository. Alternatively, you can find it at <https://unlicense.org/>.
 */

use core::{cell::Cell, marker::PhantomData};

use crate::{
    DatumBoundedPipe, DatumBoundedQueue1, DatumComposePipe, DatumDecoder, DatumOffset, DatumPipe,
    DatumPipeTokenizer, DatumResult, DatumUTF8Decoder,
};

#[cfg(feature = "alloc")]
use alloc::string::String;

#[cfg(feature = "alloc")]
use crate::{DatumParser, DatumToken, DatumValue};

// -- token outputting --

/// Char to token parsing pipeline (custom storage)
/// _Added in 1.1.0._
pub type DatumCharToTokenPipeline<B> = DatumComposePipe<DatumDecoder, DatumPipeTokenizer<B>>;

/// Byte to token parsing pipeline (custom storage)
/// _Added in 1.1.0._
pub type DatumByteToTokenPipeline<B> =
    DatumComposePipe<DatumUTF8Decoder, DatumCharToTokenPipeline<B>>;

/// Byte to token parsing pipeline.
#[cfg(feature = "alloc")]
pub fn datum_byte_to_token_pipeline() -> impl DatumPipe<Input = u8, Output = DatumToken<String>> {
    DatumByteToTokenPipeline::default()
}

/// Character to token parsing pipeline.
#[cfg(feature = "alloc")]
pub fn datum_char_to_token_pipeline() -> impl DatumPipe<Input = char, Output = DatumToken<String>> {
    DatumCharToTokenPipeline::default()
}

// -- value outputting --

/// Byte to value parsing pipeline.
#[cfg(feature = "alloc")]
pub fn datum_byte_to_value_pipeline() -> impl DatumPipe<Input = u8, Output = DatumValue> {
    let tokenizer = datum_byte_to_token_pipeline();

    DatumComposePipe(tokenizer, DatumParser::default())
}

/// Char to value parsing pipeline.
#[cfg(feature = "alloc")]
pub fn datum_char_to_value_pipeline() -> impl DatumPipe<Input = char, Output = DatumValue> {
    let tokenizer = datum_char_to_token_pipeline();

    DatumComposePipe(tokenizer, DatumParser::default())
}

// -- utils --

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
pub struct DatumLineNumberTracker<'line_number, V: Copy + Into<u32>>(
    &'line_number Cell<u32>,
    PhantomData<V>,
);
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

impl<V: Copy + Into<u32>> DatumBoundedPipe for DatumLineNumberTracker<'_, V> {
    type OutputQueue = DatumBoundedQueue1<V>;
}
