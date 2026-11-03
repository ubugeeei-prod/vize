//! Independent L3 reference: the C-23 graph checker and the P3-15 scheduling
//! contract over the P3-4 trace semantics. It reads only committed pages and
//! never calls `vize_l3`, so the Rust producers are checked against a second
//! implementation of the same contract. The dependency-free modules are also
//! included by `l3_reference/kani.rs`, which proves their laws.
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "the reference reports with std strings"
)]

pub mod behavior;
pub mod checker;
pub mod checker_corpus;
pub mod expression;
pub mod folio;
pub mod iteration;
pub mod json;
pub mod lattice;
pub mod matrix;
pub mod model;
pub mod model_behavior;
mod model_events;
pub mod observation;
pub mod program;
pub mod schedule;
pub mod schedule_fixture;
pub mod trace;
pub mod validate;
pub mod values;
pub mod view;
pub mod witness;

use std::path::{Path, PathBuf};

/// The committed reference fixtures.
#[must_use]
pub fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../formal/impeto/fixtures")
}
