//! Local inspection and reporting utilities for Vize.
//!
//! Curator owns developer-facing artifacts that describe what Vize observed or
//! generated. It is intentionally workspace-local and is not part of the
//! published crate set.

extern crate alloc;

pub mod complexity;
pub mod inspector;
pub mod profile;

pub mod repro;

pub mod legacy_plan;
