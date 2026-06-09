//! Corsa integration for collecting native TypeScript diagnostics.
//!
//! This module generates virtual TypeScript from Vue SFCs and uses the Corsa
//! LSP bridge to collect type-checking diagnostics.
#![allow(clippy::disallowed_types, clippy::disallowed_methods)]

mod collect;
mod mapping;
mod message;
mod virtual_ts;

#[cfg(test)]
mod tests;
