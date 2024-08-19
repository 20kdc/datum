/*
 * datum-rs - Quick to implement S-expression format
 * Written starting in 2024 by contributors (see CREDITS.txt at repository's root)
 * To the extent possible under law, the author(s) have dedicated all copyright and related and neighboring rights to this software to the public domain worldwide. This software is distributed without any warranty.
 * A copy of the Unlicense should have been supplied as COPYING.txt in this repository. Alternatively, you can find it at <https://unlicense.org/>.
 */

use core::{convert::TryFrom, fmt::{Display, Write}, ops::Deref};

#[cfg(feature = "alloc")]
use alloc::string::String;

use crate::{datum_error, DatumChar, DatumCharClass, DatumCharEmit, DatumError, DatumErrorKind, DatumPipe, DatumPushable, DatumResult, DatumTokenType, DatumTokenizer, DatumTokenizerAction};

/// Datum token with integrated string.
/// Notably, integer/float are stored as their values here to prevent unwritable values existing.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum DatumToken<B: Deref<Target = str>> {
    /// String. Buffer contents are the unescaped string contents.
    String(B),
    /// ID. Buffer contents are the symbol.
    ID(B),
    /// Special ID. Buffer contents are the symbol (text after, but not including, '#').
    SpecialID(B),
    /// Integer.
    Integer(i64),
    /// Float.
    Float(f64),
    ListStart,
    ListEnd
}

impl<B: Deref<Target = str>> Default for DatumToken<B> {
    fn default() -> Self {
        // arbitrarily chosen
        // this is array filler
        Self::ListEnd
    }
}

impl<B: Deref<Target = str>> TryFrom<(DatumTokenType, B)> for DatumToken<B> {
    type Error = DatumError;
    fn try_from(value: (DatumTokenType, B)) -> Result<Self, Self::Error> {
        match value {
            (DatumTokenType::String, v) => Ok(DatumToken::String(v)),
            (DatumTokenType::ID, v) => Ok(DatumToken::ID(v)),
            (DatumTokenType::SpecialID, v) => Ok(DatumToken::SpecialID(v)),
            (DatumTokenType::Numeric, v) => {
                // Numbers are parsed here to ensure that all possible [DatumToken]s are writable.
                // Originally, this was offloaded to DatumAtom, but this bloated the spec and caused all sorts of problems.
                // Besides, the quicker we get rid of these things the saner the memory use is for people who use [char;16] etc...
                if let Ok(v) = v.parse() {
                    Ok(DatumToken::Integer(v))
                } else if let Ok(v) = v.parse() {
                    Ok(DatumToken::Float(v))
                } else {
                    Err(datum_error!(BadData, "bad numeric"))
                }
            },
            (DatumTokenType::ListStart, _) => Ok(DatumToken::ListStart),
            (DatumTokenType::ListEnd, _) => Ok(DatumToken::ListEnd),
        }
    }
}

impl<B: Deref<Target = str>> DatumToken<B> {
    /// Return the token type of this token.
    #[cfg(not(tarpaulin_include))]
    pub fn token_type(&self) -> DatumTokenType {
        match self {
            Self::String(_) => DatumTokenType::String,
            Self::ID(_) => DatumTokenType::ID,
            Self::SpecialID(_) => DatumTokenType::SpecialID,
            Self::Integer(_) => DatumTokenType::Numeric,
            Self::Float(_) => DatumTokenType::Numeric,
            Self::ListStart => DatumTokenType::ListStart,
            Self::ListEnd => DatumTokenType::ListEnd,
        }
    }

    /// Return the buffer of this token, if the type has one.
    #[cfg(not(tarpaulin_include))]
    pub fn buffer(&self) -> Option<&B> {
        match &self {
            Self::String(b) => Some(b),
            Self::ID(b) => Some(b),
            Self::SpecialID(b) => Some(b),
            _ => None
        }
    }

    /// Writes this value as a valid, parsable Datum token.
    pub fn write(&self, f: &mut dyn Write) -> core::fmt::Result {
        match self {
            Self::String(b) => {
                f.write_char('\"')?;
                for v in b.deref().chars() {
                    if v == '\\' {
                        f.write_char('\\')?;
                        f.write_char('\\')?;
                    } else if v == '\r' {
                        f.write_char('\\')?;
                        f.write_char('r')?;
                    } else if v == '\n' {
                        f.write_char('\\')?;
                        f.write_char('n')?;
                    } else if v == '\t' {
                        f.write_char('\\')?;
                        f.write_char('t')?;
                    } else if ('\x00'..='\x1F').contains(&v) || v == '\x7F' {
                        for c in DatumCharEmit::make_byte_hex_escape(v as u8) {
                            f.write_char(c)?;
                        }
                    } else {
                        f.write_char(v)?;
                    }
                }
                f.write_char('\"')?;
            },
            Self::ID(b) => {
                let mut chars = b.chars();
                match chars.next() {
                    Some(v) => {
                        if DatumCharClass::identify(v) == Some(DatumCharClass::Sign) {
                            match chars.next() {
                                Some(v2) => {
                                    // business as usual
                                    DatumChar::content(v).write(f)?;
                                    DatumChar::content(v2).write(f)?;
                                },
                                None => {
                                    // lone sign
                                    return f.write_char(v);
                                }
                            }
                        } else {
                            DatumChar::content(v).write(f)?;
                        }
                        for remainder in chars {
                            DatumChar::potential_identifier(remainder).write(f)?;
                        }
                    },
                    None => {
                        f.write_char('#')?;
                        f.write_char('{')?;
                        f.write_char('}')?;
                        f.write_char('#')?;
                    }
                }
            },
            Self::SpecialID(b) => {
                f.write_char('#')?;
                let chars = b.chars();
                for remainder in chars {
                    DatumChar::potential_identifier(remainder).write(f)?;
                }
            },
            Self::Integer(v) => {
                core::fmt::write(f, format_args!("{}", v))?;
            },
            Self::Float(v) => {
                if v.is_nan() {
                    f.write_str("#i+nan.0")?;
                } else if v.is_infinite() {
                    if v.is_sign_positive() {
                        f.write_str("#i+inf.0")?;
                    } else {
                        f.write_str("#i-inf.0")?;
                    }
                } else {
                    // In my defense, it was this or relying on {:?} to be stable.
                    // This is merely cursed. That is basically asking for version breakage.
                    let mut res = DatumFloatObserver(f, false);
                    core::fmt::write(&mut res, format_args!("{:?}", v))?;
                    if !res.1 {
                        // Nothing indicating this is a float, append .0
                        f.write_str(".0")?;
                    }
                }
            },
            Self::ListStart => {
                f.write_char('(')?;
            },
            Self::ListEnd => {
                f.write_char(')')?;
            }
        }
        core::fmt::Result::Ok(())
    }
}

impl<B: Deref<Target = str>> Display for DatumToken<B> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.write(f)
    }
}

/// Internal structure to determine if Rust didn't write any indicator this number is intended to be a float.
struct DatumFloatObserver<'a>(&'a mut dyn Write, bool);

impl Write for DatumFloatObserver<'_> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.bytes() {
            if c == b'.' || c == b'e' || c == b'E' {
                // Any of these three characters indicate Rust generated some kind of float.
                self.1 = true;
            }
        }
        self.0.write_str(s)
    }
}

/// Tokenizer that uses String as an internal buffer and spits out DatumToken.
/// ```
/// use datum_rs::{DatumDecoder, DatumToken, DatumStringTokenizer, DatumComposePipe, DatumPipe};
/// let mut decoder = DatumDecoder::default().compose(DatumStringTokenizer::default());
/// let mut out = Vec::new();
/// decoder.feed_iter_to_vec(&mut out, ("these become test symbols").chars(), true);
/// ```
#[derive(Clone, Default, Debug)]
pub struct DatumPipeTokenizer<B: DatumPushable<char> + Deref<Target = str> + Default>(B, DatumTokenizer);

#[cfg(feature = "alloc")]
pub type DatumStringTokenizer = DatumPipeTokenizer<String>;

impl<B: DatumPushable<char> + Deref<Target = str> + Default> DatumPipe for DatumPipeTokenizer<B> {
    type Input = DatumChar;
    type Output = DatumToken<B>;

    fn feed<F: FnMut(Self::Output) -> DatumResult<()>>(&mut self, i: Self::Input, f: &mut F) -> DatumResult<()> {
        let m0 = &mut self.0;
        self.1.feed(i.class(), &mut |v| {
            Self::transform_action(m0, i.char(), v, f)
        })
    }

    fn eof<F: FnMut(Self::Output) -> DatumResult<()>>(&mut self, f: &mut F) -> DatumResult<()> {
        let m0 = &mut self.0;
        self.1.eof(&mut |v| {
            Self::transform_action(m0, ' ', v, f)
        })
    }
}

impl<B: DatumPushable<char> + Deref<Target = str> + Default> DatumPipeTokenizer<B> {
    fn transform_action<F: FnMut(DatumToken<B>) -> DatumResult<()>>(buffer: &mut B, char: char, action: DatumTokenizerAction, f: &mut F) -> DatumResult<()> {
        match action {
            DatumTokenizerAction::Push => buffer.push(char),
            DatumTokenizerAction::Token(v) => f(DatumToken::try_from((v, core::mem::take(buffer)))?)
        }
    }
}
