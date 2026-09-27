//! The textual dump contract: every level artifact prints and parses back.
//!
//! Dumps exist for `vize dump` and observers; they are never a transport
//! between levels.
#![expect(clippy::todo, reason = "skeleton: #6833")]

use alloc::boxed::Box;
use core::fmt;

pub mod runtime;
pub mod value;

pub use runtime::{DumpPage, DumpRuntime};
pub use value::DumpValue;

/// Which form [`Dump::print`] renders.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DumpMode {
    /// Canonical text that [`Dump::parse`] reads back.
    Full,
    /// Human-oriented text; never parsed.
    Display,
}

/// A parse failure with its 1-based line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DumpError {
    /// 1-based line of the failure.
    pub line: usize,
    /// What went wrong.
    pub message: Box<str>,
}

impl fmt::Display for DumpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let _ = f;
        todo!()
    }
}

impl core::error::Error for DumpError {}

/// A level artifact that can be dumped as text and read back.
///
/// `print` in [`DumpMode::Full`] and `parse` form a round-trip pair.
pub trait Dump: Sized {
    /// Render into `w` in `mode`.
    fn print<W: fmt::Write>(&self, w: &mut W, mode: DumpMode) -> fmt::Result;

    /// Parse canonical `Full` text back into a value.
    fn parse(input: &str) -> Result<Self, DumpError>;
}
