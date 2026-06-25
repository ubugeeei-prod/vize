//! Corsa integration for collecting native TypeScript diagnostics.
//!
//! This module generates virtual TypeScript from Vue SFCs and uses the Corsa
//! LSP bridge to collect type-checking diagnostics.
#![allow(clippy::disallowed_types, clippy::disallowed_methods)]

pub(in crate::ide) mod collect;
mod collect_virtual;
mod mapping;
mod message;
mod virtual_ts;
mod virtual_ts_art;
mod virtual_ts_inline_art;

#[cfg(test)]
mod relative_import_tests;
#[cfg(test)]
mod tests;
