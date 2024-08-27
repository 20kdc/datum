/*
 * datum-rs - Quick to implement S-expression format
 * Written starting in 2024 by contributors (see CREDITS.txt at repository's root)
 * To the extent possible under law, the author(s) have dedicated all copyright and related and neighboring rights to this software to the public domain worldwide. This software is distributed without any warranty.
 * A copy of the Unlicense should have been supplied as COPYING.txt in this repository. Alternatively, you can find it at <https://unlicense.org/>.
 */

use crate::{datum_error, DatumError, DatumOffset, DatumPipe, DatumResult};

const UTF8_DECODE_BUFFER: usize = 4;

/// UTF-8 stream decoder.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct DatumUTF8Decoder {
    /// UTF-8 decoding buffer
    buffer: [u8; UTF8_DECODE_BUFFER],
    /// UTF-8 decoding buffer length
    buffer_len: u8,
}

impl DatumPipe for DatumUTF8Decoder {
    type Input = u8;
    type Output = char;

    /// Given a [u8], returns a resulting [char], if any.
    fn feed<F: FnMut(char) -> DatumResult<()>>(&mut self, at: DatumOffset, byte: Option<u8>, f: &mut F) -> DatumResult<()> {
        if let None = byte {
            return if self.buffer_len != 0 {
                Err(datum_error!(Interrupted, at, "UTF-8 sequence"))
            } else {
                Ok(())
            }
        }
        let byte = byte.unwrap();
        if self.buffer_len >= (UTF8_DECODE_BUFFER as u8) {
            // this implies a UTF-8 character kept on continuing
            // and was not recognized as valid by Rust
            Err(datum_error!(BadData, at, "overlong UTF-8 sequence"))
        } else if self.buffer_len == 0 {
            // first char of sequence, use special handling to catch errors early
            if byte <= 127 {
                // fast-path these
                f(byte as char)
            } else if (0x80..=0xBF).contains(&byte) {
                // can't start a sequence with a continuation
                Err(datum_error!(BadData, at, "continuation at start"))
            } else {
                // start bytes of multi-byte sequences
                self.buffer[0] = byte;
                self.buffer_len = 1;
                Ok(())
            }
        } else if !(0x80..=0xBF).contains(&byte) {
            // we're supposed to be adding continuations and suddenly this shows up?
            // (this path also catches if a character comes in that looks fine at a glance but from_utf8 doesn't like)
            Err(datum_error!(BadData, at, "mid-sequence start"))
        } else {
            self.buffer[self.buffer_len as usize] = byte;
            self.buffer_len += 1;
            // check it
            let res = core::str::from_utf8(&self.buffer[0..self.buffer_len as usize]);
            if let Ok(res2) = res {
                self.buffer_len = 0;
                if let Some(v) = res2.chars().next() {
                    f(v)?;
                } else {
                    unreachable!()
                }
            }
            // else, could just mean the character hasn't finished yet
            Ok(())
        }
    }
}
