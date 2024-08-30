/*
 * datum-rs - Quick to implement S-expression format
 * Written starting in 2024 by contributors (see CREDITS.txt at repository's root)
 * To the extent possible under law, the author(s) have dedicated all copyright and related and neighboring rights to this software to the public domain worldwide. This software is distributed without any warranty.
 * A copy of the Unlicense should have been supplied as COPYING.txt in this repository. Alternatively, you can find it at <https://unlicense.org/>.
 */

use crate::{datum_error, DatumChar, DatumError, DatumOffset, DatumPipe, DatumResult};

/// Decoder's state machine
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum DatumDecoderState {
    Normal,
    Escaping(DatumOffset),
    HexEscape(DatumOffset, u32),
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

    fn feed<F: FnMut(DatumOffset, DatumChar) -> DatumResult<()>>(
        &mut self,
        at: DatumOffset,
        char: Option<char>,
        f: &mut F,
    ) -> DatumResult<()> {
        if char.is_none() {
            return if self.0 != DatumDecoderState::Normal {
                self.0 = DatumDecoderState::Normal;
                Err(datum_error!(Interrupted, at, "decoder: interrupted"))
            } else {
                Ok(())
            };
        }
        let char = char.unwrap();
        if char == '\r' {
            return Ok(());
        }
        let new_state = match self.0 {
            DatumDecoderState::Normal => {
                if char == '\\' {
                    Ok(DatumDecoderState::Escaping(at))
                } else {
                    match DatumChar::identify(char) {
                        Some(v) => {
                            f(at, v)?;
                            Ok(DatumDecoderState::Normal)
                        }
                        None => Err(datum_error!(BadData, at, "decoder: forbidden character")),
                    }
                }
            }
            DatumDecoderState::Escaping(start) => match char {
                'r' => {
                    f(start, DatumChar::content('\r'))?;
                    Ok(DatumDecoderState::Normal)
                }
                'n' => {
                    f(start, DatumChar::content('\n'))?;
                    Ok(DatumDecoderState::Normal)
                }
                't' => {
                    f(start, DatumChar::content('\t'))?;
                    Ok(DatumDecoderState::Normal)
                }
                'x' => Ok(DatumDecoderState::HexEscape(start, 0)),
                '\n' => Err(datum_error!(
                    BadData,
                    at,
                    "decoder: newline in escape sequence"
                )),
                _ => {
                    f(start, DatumChar::content(char))?;
                    Ok(DatumDecoderState::Normal)
                }
            },
            DatumDecoderState::HexEscape(start, v) => {
                if char == ';' {
                    if let Some(rustchar) = char::from_u32(v) {
                        f(start, DatumChar::content(rustchar))?;
                        Ok(DatumDecoderState::Normal)
                    } else {
                        Err(datum_error!(
                            BadData,
                            at,
                            "decoder: invalid unicode in hex escape"
                        ))
                    }
                } else {
                    let mut v_new = v;
                    v_new <<= 4;
                    if let Some(digit) = char.to_digit(16) {
                        v_new |= digit;
                        Ok(DatumDecoderState::HexEscape(start, v_new))
                    } else {
                        Err(datum_error!(BadData, at, "decoder: invalid hex digit"))
                    }
                }
            }
        }?;
        self.0 = new_state;
        Ok(())
    }
}
