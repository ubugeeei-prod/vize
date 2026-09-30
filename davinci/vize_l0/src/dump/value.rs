//! Leaf values inside a dump page.
#![expect(clippy::todo, reason = "skeleton: #6833")]

use core::fmt;

use super::DumpError;

/// A value printed inline in a dump line and parsed back from it.
pub trait DumpValue: Sized {
    /// Print the value's canonical text.
    fn print_value<W: fmt::Write>(&self, w: &mut W) -> fmt::Result;

    /// Parse the value from its canonical text found on `line`.
    fn parse_value(text: &str, line: usize) -> Result<Self, DumpError>;
}

impl DumpValue for u32 {
    fn print_value<W: fmt::Write>(&self, w: &mut W) -> fmt::Result {
        let _ = w;
        todo!()
    }

    fn parse_value(text: &str, line: usize) -> Result<Self, DumpError> {
        let _ = (text, line);
        todo!()
    }
}
