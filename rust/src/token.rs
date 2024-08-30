/*
 * datum-rs - Quick to implement S-expression format
 * Written starting in 2024 by contributors (see CREDITS.txt at repository's root)
 * To the extent possible under law, the author(s) have dedicated all copyright and related and neighboring rights to this software to the public domain worldwide. This software is distributed without any warranty.
 * A copy of the Unlicense should have been supplied as COPYING.txt in this repository. Alternatively, you can find it at <https://unlicense.org/>.
 */

use core::{
    convert::TryFrom,
    fmt::{Display, Write},
    ops::Deref,
};

#[cfg(feature = "alloc")]
use alloc::string::String;

use crate::{
    datum_error, DatumChar, DatumCharClass, DatumError, DatumOffset, DatumPipe, DatumResult,
    DatumTokenType, DatumTokenizer, DatumTokenizerAction,
};

/// Datum token with integrated string.
/// Notably, integer/float are stored as their values here to prevent unwritable values existing.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum DatumToken<B: Deref<Target = str>> {
    /// String. Buffer contents are the unescaped string contents.
    String(DatumOffset, B),
    /// Symbol. Buffer contents are the symbol.
    Symbol(DatumOffset, B),
    /// Special ID. Buffer contents are the symbol (text after, but not including, '#').
    SpecialID(DatumOffset, B),
    /// Integer.
    Integer(DatumOffset, i64),
    /// Float.
    Float(DatumOffset, f64),
    ListStart(DatumOffset),
    ListEnd(DatumOffset),
}

impl<B: Deref<Target = str>> Default for DatumToken<B> {
    fn default() -> Self {
        // arbitrarily chosen
        // this is array filler
        Self::ListEnd(0)
    }
}

impl<B: Deref<Target = str>> TryFrom<(DatumTokenType, DatumOffset, B)> for DatumToken<B> {
    type Error = DatumError;
    fn try_from(value: (DatumTokenType, DatumOffset, B)) -> Result<Self, Self::Error> {
        match value {
            (DatumTokenType::String, at, v) => Ok(DatumToken::String(at, v)),
            (DatumTokenType::Symbol, at, v) => Ok(DatumToken::Symbol(at, v)),
            (DatumTokenType::SpecialID, at, v) => Ok(DatumToken::SpecialID(at, v)),
            (DatumTokenType::Numeric, at, v) => {
                // Numbers are parsed here to ensure that all possible [DatumToken]s are writable.
                // Originally, this was offloaded to DatumAtom, but this bloated the spec and caused all sorts of problems.
                // Having to figure out how to make it reasonably safe if someone tries to make "ABCD" a numeric token did not end well.
                // Besides, the quicker we get rid of these things the saner the memory use is for people who use [char;16] etc...
                if let Ok(v) = v.parse() {
                    Ok(DatumToken::Integer(at, v))
                } else if let Ok(v) = v.parse() {
                    Ok(DatumToken::Float(at, v))
                } else {
                    Err(datum_error!(BadData, at, "token2: bad numeric"))
                }
            }
            (DatumTokenType::ListStart, at, _) => Ok(DatumToken::ListStart(at)),
            (DatumTokenType::ListEnd, at, _) => Ok(DatumToken::ListEnd(at)),
        }
    }
}

impl<B: Deref<Target = str>> DatumToken<B> {
    /// Return the token type of this token.
    #[cfg(not(tarpaulin_include))]
    pub fn token_type(&self) -> DatumTokenType {
        match self {
            Self::String(_, _) => DatumTokenType::String,
            Self::Symbol(_, _) => DatumTokenType::Symbol,
            Self::SpecialID(_, _) => DatumTokenType::SpecialID,
            Self::Integer(_, _) => DatumTokenType::Numeric,
            Self::Float(_, _) => DatumTokenType::Numeric,
            Self::ListStart(_) => DatumTokenType::ListStart,
            Self::ListEnd(_) => DatumTokenType::ListEnd,
        }
    }

    /// Return the buffer of this token, if the type has one.
    #[cfg(not(tarpaulin_include))]
    pub fn buffer(&self) -> Option<&B> {
        match self {
            Self::String(_, b) => Some(b),
            Self::Symbol(_, b) => Some(b),
            Self::SpecialID(_, b) => Some(b),
            _ => None,
        }
    }

    /// Return the offset of this token.
    #[cfg(not(tarpaulin_include))]
    pub fn offset(&self) -> DatumOffset {
        match self {
            Self::String(at, _) => *at,
            Self::Symbol(at, _) => *at,
            Self::SpecialID(at, _) => *at,
            Self::Integer(at, _) => *at,
            Self::Float(at, _) => *at,
            Self::ListStart(at) => *at,
            Self::ListEnd(at) => *at,
        }
    }

    /// Writes this value as a valid, parsable Datum token.
    pub fn write(&self, f: &mut dyn Write) -> core::fmt::Result {
        match self {
            Self::String(_, b) => {
                f.write_char('\"')?;
                for v in b.deref().chars() {
                    DatumChar::string_content(v).write(f)?;
                }
                f.write_char('\"')
            }
            Self::Symbol(_, b) => {
                let mut chars = b.chars();
                match chars.next() {
                    Some(v) => {
                        if DatumCharClass::identify(v) == Some(DatumCharClass::Sign) {
                            match chars.next() {
                                Some(v2) => {
                                    // business as usual
                                    DatumChar::content(v).write(f)?;
                                    DatumChar::content(v2).write(f)?;
                                }
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
                        core::fmt::Result::Ok(())
                    }
                    None => f.write_str("#{}#"),
                }
            }
            Self::SpecialID(_, b) => {
                f.write_char('#')?;
                let chars = b.chars();
                for remainder in chars {
                    DatumChar::potential_identifier(remainder).write(f)?;
                }
                core::fmt::Result::Ok(())
            }
            Self::Integer(_, v) => core::fmt::write(f, format_args!("{}", v)),
            Self::Float(_, v) => {
                if v.is_nan() {
                    f.write_str("#i+nan.0")
                } else if v.is_infinite() {
                    if v.is_sign_positive() {
                        f.write_str("#i+inf.0")
                    } else {
                        f.write_str("#i-inf.0")
                    }
                } else {
                    // In my defense, it was this or relying on {:?} to be stable.
                    // This is merely cursed. That is basically asking for version breakage.
                    let mut res = DatumFloatObserver(f, false);
                    core::fmt::write(&mut res, format_args!("{}", v))?;
                    if !res.1 {
                        // Nothing indicating this is a float, append .0
                        f.write_str(".0")?;
                    }
                    core::fmt::Result::Ok(())
                }
            }
            Self::ListStart(_) => f.write_char('('),
            Self::ListEnd(_) => f.write_char(')'),
        }
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
/// use datum::{DatumDecoder, DatumToken, DatumStringTokenizer, DatumComposePipe, DatumPipe};
/// let mut decoder = DatumDecoder::default().compose(DatumStringTokenizer::default());
/// let mut out = Vec::new();
/// decoder.feed_iter_to_vec(&mut out, ("these become test symbols").chars(), true);
/// ```
#[derive(Clone, Default, Debug)]
pub struct DatumPipeTokenizer<B: Write + Deref<Target = str> + Default>(B, DatumTokenizer);

#[cfg(feature = "alloc")]
pub type DatumStringTokenizer = DatumPipeTokenizer<String>;

impl<B: Write + Deref<Target = str> + Default> DatumPipe for DatumPipeTokenizer<B> {
    type Input = DatumChar;
    type Output = DatumToken<B>;

    fn feed<F: FnMut(DatumOffset, Self::Output) -> DatumResult<()>>(
        &mut self,
        at: DatumOffset,
        i: Option<Self::Input>,
        f: &mut F,
    ) -> DatumResult<()> {
        let m0 = &mut self.0;
        self.1.feed(at, i, &mut |offset, action| match action {
            DatumTokenizerAction::Push(chr) => m0.write_char(chr).map_err(|_| {
                datum_error!(OutOfRoom, at, "token2: failed to write to token buffer")
            }),
            DatumTokenizerAction::Token(v) => {
                f(offset, DatumToken::try_from((v, at, core::mem::take(m0)))?)
            }
        })
    }
}
