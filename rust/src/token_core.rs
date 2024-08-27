/*
 * datum-rs - Quick to implement S-expression format
 * Written starting in 2024 by contributors (see CREDITS.txt at repository's root)
 * To the extent possible under law, the author(s) have dedicated all copyright and related and neighboring rights to this software to the public domain worldwide. This software is distributed without any warranty.
 * A copy of the Unlicense should have been supplied as COPYING.txt in this repository. Alternatively, you can find it at <https://unlicense.org/>.
 */

use crate::{datum_error, DatumCharClass, DatumError, DatumOffset, DatumPipe, DatumResult};

/// Datum token type.
/// This is paired with the token contents, if any.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DatumTokenType {
    /// String. Buffer contents are the unescaped string contents.
    String,
    /// ID. Buffer contents are the symbol.
    ID,
    /// Special ID. Buffer contents are the symbol (text after, but not including, '#').
    SpecialID,
    /// Numeric
    Numeric,
    /// List start. Buffer is empty.
    ListStart,
    /// List end. Buffer is empty.
    ListEnd
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum DatumTokenizerState {
    /// start of whitespace_skipping block
    Start,
    /// comment block, not_newline
    LineComment,
    /// string block, not_"
    String,
    /// potential identifier block (immediate, if-expanded)
    PotentialIdentifier(DatumTokenType, DatumTokenType),
}

/// Action output by the tokenizer.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DatumTokenizerAction {
    /// Push this character to buffer.
    /// Pushes will always be contiguous.
    Push,
    /// Take token, then clear buffer.
    Token(DatumTokenType)
}

/// Datum tokenizer state machine.
/// This API is a little harder to use, but allows complete control over buffer allocation/etc.
/// In particular, it works with char classes, and expects you to keep track of bytes it sends your way with the [DatumTokenizerAction::Push] action.
/// When a token is complete, you will receive the [DatumTokenizerAction::Token] action.
/// You should also call feed with [None] when relevant to get any token at the very end of the file.
/// ```
/// use datum_rs::{DatumDecoder, DatumPipe, DatumTokenizer, DatumTokenizerAction, DatumTokenType};
/// let example = "some-symbol ; ignored comment";
/// let mut decoder = DatumDecoder::default();
/// let mut tokenizer = DatumTokenizer::default();
/// // use u8 for example's sake since we know this is all ASCII
/// // in practice you'd either use String or a proper on-stack string library
/// let mut token: [u8; 11] = [0; 11];
/// let mut token_len: usize = 0;
/// for b in example.chars() {
///     decoder.feed(0, Some(b), &mut |c| {
///         // note the error from one stage can be passed to the previous
///         tokenizer.feed(0, Some(c.class()), &mut |a| {
///             match a {
///                 DatumTokenizerAction::Push => {
///                     token[token_len] = c.char() as u8;
///                     token_len += 1;
///                 },
///                 DatumTokenizerAction::Token(tt) => {
///                     // Example 'parser': only accepts sequences of this one symbol
///                     assert_eq!(tt, DatumTokenType::ID);
///                     assert_eq!(&token[..token_len], b"some-symbol");
///                     token_len = 0;
///                 }
///             }
///             Ok(())
///         })
///     }).unwrap();
/// }
/// // At the end, you have to process EOF, etc.
/// // If you're really in a rush, adding a single newline to the end should work
/// // That said, if you do it, you keep the pieces (particularly re: unterminated strings!)
/// decoder.feed(0, None, &mut |_| {Ok(())}).unwrap();
/// tokenizer.feed(0, None, &mut |a| {
///     match a {
///         DatumTokenizerAction::Push => {},
///         DatumTokenizerAction::Token(tt) => {
///             assert_eq!(tt, DatumTokenType::ID);
///             assert_eq!(&token[..token_len], b"some-symbol");
///             token_len = 0;
///         },
///     }
///     Ok(())
/// }).unwrap();
/// ```

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct DatumTokenizer(DatumTokenizerState, bool);

impl Default for DatumTokenizer {
    fn default() -> Self {
        Self(DatumTokenizerState::Start, false)
    }
}

impl DatumPipe for DatumTokenizer {
    // this is a bit awkward since it has to be kept in sync
    // ...which doesn't really fit the DatumPipe style!
    /// but it should work
    type Input = DatumCharClass;
    type Output = DatumTokenizerAction;

    /// Given an incoming character class, returns the resulting actions.
    fn feed<F: FnMut(DatumTokenizerAction) -> DatumResult<()>>(&mut self, at: DatumOffset, class: Option<DatumCharClass>, f: &mut F) -> DatumResult<()> {
        if let None = class {
            self.0 = match self.0 {
                DatumTokenizerState::Start => Ok(DatumTokenizerState::Start),
                DatumTokenizerState::LineComment => Ok(DatumTokenizerState::Start),
                DatumTokenizerState::String => Err(datum_error!(Interrupted, at, "mid-string eof")),
                DatumTokenizerState::PotentialIdentifier(immediate, _) => {
                    f(DatumTokenizerAction::Token(immediate))?;
                    Ok(DatumTokenizerState::Start)
                }
            }?;
            return Ok(())
        }
        let class = class.unwrap();
        self.0 = match self.0 {
            DatumTokenizerState::Start => Self::start_feed(f, class),
            DatumTokenizerState::LineComment => {
                if class == DatumCharClass::Newline {
                    Ok(DatumTokenizerState::Start)
                } else {
                    Ok(DatumTokenizerState::LineComment)
                }
            },
            DatumTokenizerState::String => {
                if class == DatumCharClass::String {
                    f(DatumTokenizerAction::Token(DatumTokenType::String))?;
                    Ok(DatumTokenizerState::Start)
                } else {
                    f(DatumTokenizerAction::Push)?;
                    Ok(DatumTokenizerState::String)
                }
            },
            DatumTokenizerState::PotentialIdentifier(immediate, expanded) => {
                if class.potential_identifier() {
                    f(DatumTokenizerAction::Push)?;
                    Ok(DatumTokenizerState::PotentialIdentifier(expanded, expanded))
                } else {
                    f(DatumTokenizerAction::Token(immediate))?;
                    Self::start_feed(f, class)
                }
            },
        }?;
        Ok(())
    }
}

impl DatumTokenizer {
    /// Handling for the start state.
    /// This is used both in that state and when going 'through' that state when leaving another state.
    fn start_feed<F: FnMut(DatumTokenizerAction) -> DatumResult<()>>(f: &mut F, class: DatumCharClass) -> DatumResult<DatumTokenizerState> {
        match class {
            DatumCharClass::Content => {
                f(DatumTokenizerAction::Push)?;
                Ok(DatumTokenizerState::PotentialIdentifier(DatumTokenType::ID, DatumTokenType::ID))
            },
            DatumCharClass::Whitespace => Ok(DatumTokenizerState::Start),
            DatumCharClass::Newline => Ok(DatumTokenizerState::Start),
            DatumCharClass::LineComment => Ok(DatumTokenizerState::LineComment),
            DatumCharClass::String => Ok(DatumTokenizerState::String),
            DatumCharClass::ListStart => {
                f(DatumTokenizerAction::Token(DatumTokenType::ListStart))?;
                Ok(DatumTokenizerState::Start)
            },
            DatumCharClass::ListEnd => {
                f(DatumTokenizerAction::Token(DatumTokenType::ListEnd))?;
                Ok(DatumTokenizerState::Start)
            },
            DatumCharClass::SpecialID => {
                Ok(DatumTokenizerState::PotentialIdentifier(DatumTokenType::SpecialID, DatumTokenType::SpecialID))
            },
            DatumCharClass::Sign => {
                f(DatumTokenizerAction::Push)?;
                Ok(DatumTokenizerState::PotentialIdentifier(DatumTokenType::ID, DatumTokenType::Numeric))
            },
            DatumCharClass::Digit => {
                f(DatumTokenizerAction::Push)?;
                Ok(DatumTokenizerState::PotentialIdentifier(DatumTokenType::Numeric, DatumTokenType::Numeric))
            },
        }
    }
}
