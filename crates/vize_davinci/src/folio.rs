//! Folio - the textual dump format every Davinci stage speaks.
//!
//! A folio is a stage artifact rendered as text. The contract is defined by
//! [`Folio`] together with the mode-explicit equality laws below; the
//! normalization rules shared by every folio page are documented in
//! `davinci-road/plan/folio-format.md`.
//!
//! # Equality laws (mode-explicit)
//!
//! - [`FolioMode::Full`] is the injective, parseable form. The round-trip
//!   laws apply: `print(parse(t)) == t` byte-for-byte for canonical
//!   (normalized) text `t`, and `parse(print(v)) == v` structurally for
//!   values. Non-canonical input is normalized by the first print, by
//!   design - parse is lenient about section order and unsorted lists;
//!   print always emits the canonical form.
//! - [`FolioMode::Display`] elides spans and defaults for humans and
//!   carries **no** round-trip law. Do not parse `Display` output.
//!
//! Normalization (stable sequential ids, sorted map iteration) applies to
//! both modes.

use core::fmt;

use vize_s0::String;

pub mod croquis;
pub mod dump;
pub mod feed;
pub mod page;
pub mod repro;
pub mod value;

/// `#[derive(Folio)]` - generates the mechanical trio (print / parse /
/// field order) for an owned document struct; see `vize_davinci_derive`.
/// Anything semantic (`Display` elision, section meaning) stays
/// hand-written, so derived pages print the same canonical text in both
/// modes.
pub use vize_davinci_derive::Folio;

/// Print mode for a folio page.
///
/// See the module docs for the mode-explicit equality laws.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FolioMode {
    /// Injective, parseable form. Round-trip laws apply.
    Full,
    /// Human-oriented form: spans and defaults are elided. No round-trip
    /// law.
    Display,
}

/// Parse error for a folio page.
///
/// Carries the 1-based line number in the input at which parsing failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolioError {
    /// 1-based line number of the offending line (0 when the error is not
    /// attributable to a single line, e.g. unexpected end of input).
    pub line: usize,
    /// Human-readable description of the failure.
    pub message: String,
}

impl FolioError {
    /// Create an error attributed to `line` (1-based).
    pub fn new(line: usize, message: String) -> Self {
        Self { line, message }
    }
}

impl fmt::Display for FolioError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.line == 0 {
            write!(f, "folio parse error: {}", self.message)
        } else {
            write!(
                f,
                "folio parse error at line {}: {}",
                self.line, self.message
            )
        }
    }
}

impl core::error::Error for FolioError {}

/// A stage artifact that can be dumped as text and read back.
///
/// `print` with [`FolioMode::Full`] and `parse` form the round-trip pair;
/// see the module docs for the exact laws. Both directions normalize:
/// `parse` accepts non-canonical input where the format allows it, and
/// `print` always emits canonical text.
pub trait Folio: Sized {
    /// Render this value into `w` in the requested mode.
    ///
    /// `Full` output is canonical: printing the same value twice yields
    /// identical bytes, and distinct values yield distinct bytes.
    fn print<W: fmt::Write>(&self, w: &mut W, mode: FolioMode) -> fmt::Result;

    /// Parse canonical or near-canonical `Full`-mode text back into a
    /// value.
    ///
    /// Never call this on `Display`-mode output.
    fn parse(input: &str) -> Result<Self, FolioError>;

    /// Convenience: render into an owned string.
    ///
    /// Infallible because writing into a growable string cannot fail.
    fn print_to_string(&self, mode: FolioMode) -> String {
        let mut out = String::default();
        self.print(&mut out, mode)
            .expect("printing a folio into a string cannot fail");
        out
    }
}

/// Snapshot-assert a [`Folio`] value using the normalized `Full`-mode
/// printer.
///
/// Expands to `insta::assert_snapshot!` over
/// `Folio::print_to_string(&value, FolioMode::Full)`, so snapshots pin the
/// canonical text and stay diffable. The expansion carries the workspace's
/// `#[allow(clippy::disallowed_macros)]` insta convention; the consuming
/// crate needs `insta` as a dev-dependency.
#[macro_export]
macro_rules! assert_folio_snapshot {
    ($value:expr) => {{
        #[allow(clippy::disallowed_macros)]
        {
            ::insta::assert_snapshot!($crate::folio::Folio::print_to_string(
                &$value,
                $crate::folio::FolioMode::Full,
            ));
        }
    }};
}
