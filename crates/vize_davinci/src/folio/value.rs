//! [`FolioValue`] - how a single field value prints and parses on a derived
//! page.
//!
//! `#[derive(Folio)]` routes every scalar line, list entry, map key and map
//! value through this trait, so the set of impls below *is* the set of
//! supported field value types. An unsupported type is a missing-impl
//! compile error in the deriving crate - never a silent guess at a format.
//!
//! Values are line-atomic: a value containing `\n` would span physical
//! lines and break the byte round-trip, so embedding one is outside the
//! contract (`davinci-road/plan/folio-format.md`, "Derived pages"). The
//! same goes for a map key containing `=`.

use core::fmt;

use vize_s0::{String, cstr};

use super::FolioError;

/// One field value on a derived folio page.
///
/// `print_value` must be canonical (same value, same bytes) and
/// `parse_value(print_value(v)) == v` must hold; `parse_value` may be
/// lenient toward non-canonical spellings (e.g. `007`), which the first
/// print then normalizes - the same leniency contract the page level has.
pub trait FolioValue: Sized {
    /// Write the canonical text of this value.
    ///
    /// # Errors
    ///
    /// Propagates the writer's failure.
    fn print_value<W: fmt::Write>(&self, w: &mut W) -> fmt::Result;

    /// Read a value back from the text of one line (or one side of a map
    /// entry). `line` is the 1-based line number for the error.
    ///
    /// # Errors
    ///
    /// Returns a [`FolioError`] naming the offending text and line.
    fn parse_value(text: &str, line: usize) -> Result<Self, FolioError>;
}

impl FolioValue for bool {
    fn print_value<W: fmt::Write>(&self, w: &mut W) -> fmt::Result {
        write!(w, "{self}")
    }

    fn parse_value(text: &str, line: usize) -> Result<Self, FolioError> {
        match text {
            "true" => Ok(true),
            "false" => Ok(false),
            _ => Err(FolioError::new(line, cstr!("invalid bool `{text}`"))),
        }
    }
}

impl FolioValue for String {
    fn print_value<W: fmt::Write>(&self, w: &mut W) -> fmt::Result {
        w.write_str(self.as_str())
    }

    fn parse_value(text: &str, _line: usize) -> Result<Self, FolioError> {
        Ok(String::from(text))
    }
}

macro_rules! impl_folio_value_for_int {
    ($($int:ty),* $(,)?) => {$(
        impl FolioValue for $int {
            fn print_value<W: fmt::Write>(&self, w: &mut W) -> fmt::Result {
                write!(w, "{self}")
            }

            fn parse_value(text: &str, line: usize) -> Result<Self, FolioError> {
                text.parse::<$int>()
                    .map_err(|_| FolioError::new(line, cstr!("invalid integer `{text}`")))
            }
        }
    )*};
}

impl_folio_value_for_int!(
    u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize
);
