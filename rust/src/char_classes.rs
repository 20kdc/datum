/*
 * gabien-datum-rs - Quick to implement S-expression format
 * Written starting in 2024 by contributors (see CREDITS.txt at repository's root)
 * To the extent possible under law, the author(s) have dedicated all copyright and related and neighboring rights to this software to the public domain worldwide. This software is distributed without any warranty.
 * A copy of the Unlicense should have been supplied as COPYING.txt in this repository. Alternatively, you can find it at <https://unlicense.org/>.
 */

use core::{fmt::{Display, Write}, ops::Deref};

/// Datum character class.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DatumCharClass {
    /// Escaped characters, anything else
    Content,
    /// 0-32 and 127 but not 10
    Whitespace,
    /// 10
    Newline,
    /// ';'
    LineComment,
    /// '"'
    String,
    /// '('
    ListStart,
    /// ')'
    ListEnd,
    /// '#'
    SpecialID,
    /// '-'
    Sign,
    /// '0' - '9'
    Digit,
}

impl DatumCharClass {
    /// If this character class is a potential identifier.
    /// Note that this can be accessed via [DatumChar] via [DatumChar::deref].
    /// ```
    /// use datum_rs::DatumChar;
    /// assert!(DatumChar::identify('a').expect("not backslash").potential_identifier());
    /// ```
    #[inline]
    pub const fn potential_identifier(&self) -> bool {
        matches!(self, Self::Content | Self::Sign | Self::Digit | Self::SpecialID)
    }

    /// If this character class starts a number.
    /// Note that this can be accessed via [DatumChar] via [DatumChar::deref].
    /// ```
    /// use datum_rs::DatumChar;
    /// assert!(DatumChar::identify('0').expect("not backslash").numeric_start());
    /// ```
    #[inline]
    pub const fn numeric_start(&self) -> bool {
        matches!(self, Self::Sign | Self::Digit)
    }

    /// Identifies a character.
    /// Meta-class characters return [None].
    pub const fn identify(v: char) -> Option<Self> {
        if v == '\n' {
            Some(DatumCharClass::Newline)
        } else if v == '\t' || v == ' ' {
            // important that this comes before the meta-class check
            Some(DatumCharClass::Whitespace)
        } else if v < ' ' || v == '\x7F' || v == '\\' {
            None
        } else if v == ';' {
            Some(DatumCharClass::LineComment)
        } else if v == '"' {
            Some(DatumCharClass::String)
        } else if v == '(' {
            Some(DatumCharClass::ListStart)
        } else if v == ')' {
            Some(DatumCharClass::ListEnd)
        } else if v == '#' {
            Some(DatumCharClass::SpecialID)
        } else if v == '-' {
            Some(DatumCharClass::Sign)
        } else if v >= '0' && v <= '9' {
            Some(DatumCharClass::Digit)
        } else {
            Some(DatumCharClass::Content)
        }
    }
}

/// Datum character with class.
/// It is not possible to create an instance of this enum which cannot be emitted.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct DatumChar {
    /// The raw value of this character.
    char: char,
    /// The class of this character.
    class: DatumCharClass
}

/// How to emit a class/char pair reliably.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct DatumCharEmit(u8, [char;5]);

impl Deref for DatumCharEmit {
    type Target = [char];
    fn deref(&self) -> &Self::Target {
        &self.1[..(self.0 as usize)]
    }
}

impl From<DatumChar> for DatumCharEmit {
    fn from(value: DatumChar) -> Self {
        let v = value.char;
        if value.class == DatumCharClass::Content {
            if v == '\n' {
                Self(2, ['\\', 'n', '\x00', '\x00', '\x00'])
            } else if v == '\r' {
                Self(2, ['\\', 'r', '\x00', '\x00', '\x00'])
            } else if v == '\t' {
                Self(2, ['\\', 't', '\x00', '\x00', '\x00'])
            } else if ((v as u32) < 32) || v == '\x1F' {
                Self(5, Self::make_byte_hex_escape(v as u8))
            } else {
                match DatumCharClass::identify(v) {
                    Some(DatumCharClass::Content) => Self(1, [v, '\x00', '\x00', '\x00', '\x00']),
                    _ => Self(2, ['\\', v, '\x00', '\x00', '\x00'])
                }
            }
        } else {
            // if the char was identified as this type it's self-identifying
            Self(1, [v, '\x00', '\x00', '\x00', '\x00'])
        }
    }
}

impl DatumCharEmit {
    const fn make_hex_digit(v: u8) -> char {
        if v >= 0xA {
            (('a' as u8) + (v - 0xA)) as char
        } else {
            (('0' as u8) + v) as char
        }
    }

    /// Creates a hex escape for a byte.
    pub const fn make_byte_hex_escape(v: u8) -> [char; 5] {
        ['\\', 'x', Self::make_hex_digit(v >> 4), Self::make_hex_digit(v & 0xF), ';']
    }

    /// Write the necessary UTF-8 characters that will be read back as the source [DatumChar].
    pub fn write(&self, f: &mut dyn Write) -> core::fmt::Result {
        if self.0 >= 1 {
            f.write_char(self.1[0])?;
        }
        if self.0 >= 2 {
            f.write_char(self.1[1])?;
        }
        if self.0 >= 3 {
            f.write_char(self.1[2])?;
        }
        if self.0 >= 4 {
            f.write_char(self.1[3])?;
        }
        if self.0 >= 5 {
            f.write_char(self.1[4])?;
        }
        // never exceeds this, so avoid unnecessary bounds-checking
        core::fmt::Result::Ok(())
    }
}

impl DatumChar {
    /// Returns the byte in Datum's class/byte stream.
    /// ```
    /// use datum_rs::DatumChar;
    /// let list_start = DatumChar::identify('(').expect("not backslash");
    /// assert_eq!(list_start.char(), '(');
    /// ```
    #[inline]
    pub const fn char(&self) -> char {
        self.char
    }

    /// Returns the class in Datum's class/byte stream.
    /// ```
    /// use datum_rs::{DatumChar, DatumCharClass};
    /// let list_start = DatumChar::identify('(').expect("not backslash");
    /// assert_eq!(list_start.class(), DatumCharClass::ListStart);
    /// ```
    #[inline]
    pub const fn class(&self) -> DatumCharClass {
        self.class
    }

    /// Returns how to emit the character.
    /// ```
    /// use datum_rs::{DatumChar, DatumCharClass};
    /// let content_open_paren = DatumChar::content('(');
    /// assert_eq!(*content_open_paren.emit(), ['\\', '(']);
    /// ```
    #[inline]
    pub fn emit(&self) -> DatumCharEmit {
        DatumCharEmit::from(*self)
    }

    /// Write the necessary UTF-8 characters that will be read back as this [DatumChar].
    pub fn write(&self, f: &mut dyn Write) -> core::fmt::Result {
        self.emit().write(f)
    }

    /// Identifies an unescaped character and returns the corresponding [DatumChar].
    /// Backslash is special due to being the escape character, and this will return [None].
    /// ```
    /// use datum_rs::DatumChar;
    /// assert_eq!(DatumChar::identify('\\'), None);
    /// assert_ne!(DatumChar::identify('a'), None);
    /// ```
    #[inline]
    pub const fn identify(v: char) -> Option<DatumChar> {
        match DatumCharClass::identify(v) {
            None => None,
            Some(class) => Some(DatumChar { char: v, class })
        }
    }

    /// Creates a content character for the given value.
    pub const fn content(v: char) -> DatumChar {
        DatumChar { char: v, class: DatumCharClass::Content }
    }

    /// Creates a potential identifier character for the given value.
    pub const fn potential_identifier(v: char) -> DatumChar {
        match Self::identify(v) {
            None => Self::content(v),
            Some(rchr) => if rchr.class().potential_identifier() {
                rchr
            } else {
                Self::content(v)
            }
        }
    }
}

impl Display for DatumChar {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.write(f)
    }
}

impl Deref for DatumChar {
    type Target = DatumCharClass;
    fn deref(&self) -> &Self::Target {
        &self.class
    }
}

impl Default for DatumChar {
    fn default() -> Self {
        // Whitespace ' ' should avoid messing up whatever somehow receives this value.
        DatumChar { char: ' ', class: DatumCharClass::Whitespace }
    }
}
