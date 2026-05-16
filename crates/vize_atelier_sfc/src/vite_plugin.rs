//! Native request classification for the Vite plugin.
//!
//! Vite hook orchestration stays in TypeScript because it depends on Vite's
//! async resolver and plugin context. This module owns the deterministic pieces
//! that are easy to keep fast and strict in Rust: query parsing, virtual module
//! normalization, style virtual suffixes, and Vue boundary detection.

mod boundary;
mod query;
mod request;
mod style;

#[cfg(test)]
mod tests;

pub use request::{VitePluginRequest, classify_vite_plugin_request};
