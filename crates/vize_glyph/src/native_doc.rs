//! Source-borrowing formatting documents and an independent iterative printer.
//!
//! Syntax consumers construct this neutral IR; the printer never reparses
//! source or queries semantic levels. Product route integration is separate.

#[path = "native_doc/document.rs"]
mod document;
#[path = "native_doc/printer.rs"]
mod printer;

pub use document::{Doc, Line};
pub use printer::{LineEnding, PrintOptions, print};
