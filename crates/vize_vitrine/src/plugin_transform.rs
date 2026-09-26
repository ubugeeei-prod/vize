//! Bounded, versioned JS edits at the single pre-canonical L2 boundary.
//!
//! Plugins never receive Rust AST ownership. The native reader projects real
//! page IDs; validated edits update existing native attributes in the lowering
//! arena. Canonical facts are computed only after the last edit. No source is
//! regenerated or reparsed to enact a transform.
#![expect(
    clippy::disallowed_types,
    reason = "serialized plugin boundary uses std strings"
)]
#![expect(
    clippy::disallowed_macros,
    reason = "serialized plugin errors use std formatting"
)]
#![expect(
    clippy::disallowed_methods,
    reason = "serialized plugin boundary uses owned strings"
)]

mod cache;
mod compile;
mod disk;
mod edits;
mod schema;
#[cfg(test)]
mod tests;
mod walk;

pub(crate) use compile::{CompileOptions, compile};
pub(crate) use schema::Identity;

type Result<T> = std::result::Result<T, String>;
