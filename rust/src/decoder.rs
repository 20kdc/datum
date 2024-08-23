/*
 * datum-rs - Quick to implement S-expression format
 * Written starting in 2024 by contributors (see CREDITS.txt at repository's root)
 * To the extent possible under law, the author(s) have dedicated all copyright and related and neighboring rights to this software to the public domain worldwide. This software is distributed without any warranty.
 * A copy of the Unlicense should have been supplied as COPYING.txt in this repository. Alternatively, you can find it at <https://unlicense.org/>.
 */

use crate::{datum_error, DatumChar, DatumError, DatumErrorKind, DatumPipe, DatumResult};

/// Decoder's state machine
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum DatumDecoderState {
    Normal,
    Escaping,
    HexEscape(u32)
}

/// Decoder for the Datum encoding layer.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct DatumDecoder(DatumDecoderState);

impl Default for DatumDecoder {
    #[inline]
    fn default() -> DatumDecoder {
        DatumDecoder(DatumDecoderState::Normal)
    }
}

impl DatumPipe for DatumDecoder {
    type Input = char;
    type Output = DatumChar;

    fn feed<F: FnMut(DatumChar) -> DatumResult<()>>(&mut self, char: Option<char>, f: &mut F) -> DatumResult<()> {
        if let None = char {
            return if self.0 != DatumDecoderState::Normal {
                self.0 = DatumDecoderState::Normal;
                Err(datum_error!(Interrupted, "decoder: interrupted"))
            } else {
                Ok(())
            }
        }
        let char = char.unwrap();
        if char == '\r' {
            return Ok(());
        }
        let new_state = match self.0 {
            DatumDecoderState::Normal => {
                if char == '\\' {
                    Ok(DatumDecoderState::Escaping)
                } else {
                    match DatumChar::identify(char) {
                        Some(v) => {
                            f(v)?;
                            Ok(DatumDecoderState::Normal)
                        },
                        None => Err(datum_error!(BadData, "forbidden character"))
                    }
                }
            },
            DatumDecoderState::Escaping => {
                match char {
                    'r' => {
                        f(DatumChar::content('\r'))?;
                        Ok(DatumDecoderState::Normal)
                    },
                    'n' => {
                        f(DatumChar::content('\n'))?;
                        Ok(DatumDecoderState::Normal)
                    },
                    't' => {
                        f(DatumChar::content('\t'))?;
                        Ok(DatumDecoderState::Normal)
                    },
                    'x' => {
                        Ok(DatumDecoderState::HexEscape(0))
                    },
                    '\n' => {
                        Err(datum_error!(BadData, "newline in escape sequence"))
                    },
                    _ => {
                        f(DatumChar::content(char))?;
                        Ok(DatumDecoderState::Normal)
                    }
                }
            },
            DatumDecoderState::HexEscape(v) => {
                if char == ';' {
                    if let Some(rustchar) = char::from_u32(v) {
                        f(DatumChar::content(rustchar))?;
                        Ok(DatumDecoderState::Normal)
                    } else {
                        Err(datum_error!(BadData, "invalid unicode in hex escape"))
                    }
                } else {
                    let mut v_new = v;
                    v_new <<= 4;
                    if let Some(digit) = char.to_digit(16) {
                        v_new |= digit;
                        Ok(DatumDecoderState::HexEscape(v_new))
                    } else {
                        Err(datum_error!(BadData, "invalid hex digit"))
                    }
                }
            }
        }?;
        self.0 = new_state;
        Ok(())
    }
}
