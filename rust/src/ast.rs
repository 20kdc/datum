/*
 * datum-rs - Quick to implement S-expression format
 * Written starting in 2024 by contributors (see CREDITS.txt at repository's root)
 * To the extent possible under law, the author(s) have dedicated all copyright and related and neighboring rights to this software to the public domain worldwide. This software is distributed without any warranty.
 * A copy of the Unlicense should have been supplied as COPYING.txt in this repository. Alternatively, you can find it at <https://unlicense.org/>.
 */

// A heads up to future people reading this.
// This entire feature was made alloc-only because of the following error:
//
// error[E0275]: overflow evaluating the requirement `Vec<ast::DatumValue<AllocDatumTree>>: Clone`
//    --> src/tests.rs:14:153
//     |
// 14  | ...der::default(), DatumComposePipe(DatumStringTokenizer::default(), AllocDatumParser::default()));
//     |                    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
//     |
// note: required for `ast::DatumValue<AllocDatumTree>` to implement `Clone`
//    --> src/ast.rs:83:87
//     |
// 83  | impl<M: DatumTree<Buffer = B, ListRef = R>, B: Clone + Deref<Target = str>, R: Clone> Clone for DatumValue<M> {
//     |                                                                                -----  ^^^^^     ^^^^^^^^^^^^^
//     |                                                                                |
//     |                                                                                unsatisfied trait bound introduced here
//     = note: 2 redundant requirements hidden
//     = note: required for `VecDatumParserStack<AllocDatumTree>` to implement `Clone`
//
// In addition to this, the memory structure became kind of a kudzu.
// Basically, the assumption was you'd be using fixed-size buffers for your strings and then have a manager object handling list allocations, but the more I think about it the more wasteful the string handling seems.
// Also, there was a specific array needed to handle the parser's internal stack.
// That needed a layer of indirection to protect parser internal details from callers while still allowing them to supply their own allocators.
// Finally, there were a ton of cases which were like "if this specific internal array runs out of memory, bring the parser to an error state" - repeat for practically every line in the parser...
// Sorry. - 20kdc

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::{Debug, Display};
use core::hash::Hash;
use core::{convert::TryFrom, fmt::Write};

use crate::{
    datum_error, unary, DatumAtom, DatumBoundedPipe, DatumMayContainAtom, DatumOffset, DatumPipe,
    DatumResult, DatumToken, DatumTokenType, DatumWriter,
};

/// Datum AST node / value.
#[derive(Clone, PartialEq, PartialOrd, Debug)]
pub enum DatumValue {
    Atom(DatumAtom<String>),
    List(Vec<DatumValue>),
}

impl Default for DatumValue {
    // tests.rs does cover this, but it's not detected
    #[cfg(not(tarpaulin_include))]
    #[inline]
    fn default() -> Self {
        Self::Atom(DatumAtom::Nil)
    }
}

impl Hash for DatumValue {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        match self {
            Self::Atom(atm) => {
                atm.hash(state);
            }
            Self::List(vec) => {
                // **Notice: The 'type ID namespace' is shared with DatumAtom.**
                state.write_u8(6);
                vec.hash(state);
            }
        }
    }
}

impl DatumValue {
    /// Writes a value from AST.
    pub fn write_to(&self, f: &mut dyn Write, writer: &mut DatumWriter) -> core::fmt::Result {
        match self {
            DatumValue::Atom(v) => writer.write_atom(f, v),
            DatumValue::List(v) => {
                let ls: DatumToken<&str> = DatumToken::ListStart(0);
                let le: DatumToken<&str> = DatumToken::ListEnd(0);
                writer.write_token(f, &ls)?;
                for e in v {
                    e.write_to(f, writer)?;
                }
                writer.write_token(f, &le)
            }
        }
    }

    /// If this value is a list, returns a reference to it, otherwise [None].
    pub fn as_list(&self) -> Option<&Vec<DatumValue>> {
        match self {
            DatumValue::List(list) => Some(list),
            _ => None,
        }
    }
}

impl Display for DatumValue {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.write_to(f, &mut DatumWriter::default())
    }
}

impl DatumMayContainAtom<String> for DatumValue {
    fn as_atom(&self) -> Option<&DatumAtom<String>> {
        if let DatumValue::Atom(a) = self {
            Some(a)
        } else {
            None
        }
    }
}

/// Datum parser (from tokens into values).
#[derive(Clone, Debug, Default)]
pub struct DatumParser {
    start: DatumOffset,
    stack: Vec<Vec<DatumValue>>,
}

impl DatumBoundedPipe for DatumParser {
    type OutputQueueSize = unary::C1;
}

impl DatumPipe for DatumParser {
    type Input = DatumToken<String>;
    type Output = DatumValue;

    fn feed<F: FnMut(DatumOffset, DatumValue) -> DatumResult<()>>(
        &mut self,
        at: DatumOffset,
        token: Option<Self::Input>,
        f: &mut F,
    ) -> DatumResult<()> {
        if token.is_none() {
            return if !self.stack.is_empty() {
                Err(datum_error!(Interrupted, at, "eof inside list"))
            } else {
                Ok(())
            };
        }
        let token = token.unwrap();
        if self.stack.is_empty() {
            // outermost value started here
            self.start = at;
        }
        match token.token_type() {
            DatumTokenType::ListStart => {
                let list = Vec::new();
                self.stack.push(list);
                Ok(())
            }
            DatumTokenType::ListEnd => {
                let res = self.stack.pop();
                if let Some(v) = res {
                    self.feed_value(DatumValue::List(v), f)
                } else {
                    Err(datum_error!(BadData, at, "end of list while not in list"))
                }
            }
            _ => match DatumAtom::try_from(token) {
                Err(e) => Err(e),
                Ok(v) => self.feed_value(DatumValue::Atom(v), f),
            },
        }
    }
}

impl DatumParser {
    fn feed_value<F: FnMut(DatumOffset, DatumValue) -> DatumResult<()>>(
        &mut self,
        v: DatumValue,
        f: &mut F,
    ) -> DatumResult<()> {
        match self.stack.pop() {
            None => f(self.start, v),
            Some(mut list) => {
                list.push(v);
                self.stack.push(list);
                Ok(())
            }
        }
    }
}
