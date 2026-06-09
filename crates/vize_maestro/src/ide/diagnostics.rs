//! Diagnostics aggregation from multiple sources.
//!
//! Aggregates diagnostics from:
//! - SFC parser errors
//! - Template parser errors
//! - vize_patina (linter)
//! - Future: vize_canon (type checker)
#![allow(clippy::disallowed_types, clippy::disallowed_methods)]

mod builder;
mod collectors;
#[cfg(feature = "native")]
mod corsa;
mod line_index;
mod service;
mod severity;
#[cfg(test)]
mod tests;

pub use builder::DiagnosticBuilder;
pub use service::DiagnosticService;
pub use severity::Severity;

pub(in crate::ide) use line_index::LineIndex;
#[cfg(test)]
pub(in crate::ide) use line_index::offset_to_line_col;
#[cfg(feature = "native")]
pub(in crate::ide) use service::{SourceMapping, VirtualTsResult};

/// Diagnostic source identifiers.
pub mod sources {
    pub const SFC_PARSER: &str = "vize/sfc";
    pub const SFC_COMPILER: &str = "vize/sfc-compile";
    pub const TEMPLATE_PARSER: &str = "vize/template";
    pub const SCRIPT_PARSER: &str = "vize/script";
    pub const LINTER: &str = "vize/lint";
    pub const TYPE_CHECKER: &str = "vize/types";
    pub const MUSEA: &str = "vize/musea";
}
