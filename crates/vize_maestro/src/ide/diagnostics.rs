//! Diagnostics aggregation from multiple sources.
//!
//! Aggregates diagnostics from:
//! - SFC parser errors
//! - Template parser errors
//! - vize_patina (linter)
//! - Future: vize_canon (type checker)
#![allow(clippy::disallowed_types, clippy::disallowed_methods)]

#[cfg(all(test, feature = "native"))]
mod art_variant_typecheck_tests;
mod builder;
mod collectors;
mod component_props;
#[cfg(test)]
mod component_props_tests;
#[cfg(feature = "native")]
pub(in crate::ide) mod corsa;
#[cfg(all(test, feature = "native"))]
mod editor_typecheck_fixture;
#[cfg(all(test, feature = "native"))]
mod editor_typecheck_tests;
mod line_index;
mod linter_options;
mod service;
mod severity;
#[cfg(all(test, feature = "native"))]
mod typecheck_unavailable_tests;
mod vize_sfc_type;
// `insta`'s snapshot macros expand through the disallowed `std::format!`; the
// expansion is inside `insta`, so only an allow at the test module can silence
// it. See CONTRIBUTING.md, "Snapshot assertions in test targets".
#[allow(clippy::disallowed_macros)]
#[cfg(test)]
mod tests;
#[cfg(all(test, feature = "native"))]
mod vize_sfc_type_tests;

pub use builder::DiagnosticBuilder;
pub use service::DiagnosticService;
pub use severity::Severity;

pub(in crate::ide) use line_index::LineIndex;
#[cfg(test)]
pub(in crate::ide) use line_index::offset_to_line_col;
#[cfg(feature = "native")]
pub(in crate::ide) use service::{SourceMapping, VirtualTsResult};

#[cfg(feature = "native")]
pub(crate) const TYPECHECK_UNAVAILABLE_HINT_MESSAGE: &str = "Type checking is unavailable in this workspace. Make sure `tsconfig.json` exists. \
     Install `typescript@^7` or configure `typeChecker.corsaPath`; \
     see https://vizejs.dev/guide/static-analysis.";

#[cfg(feature = "native")]
pub(crate) const TYPECHECK_UNAVAILABLE_NOTICE_MESSAGE: &str = "Vize: type checking is unavailable in this workspace. Make sure tsconfig.json exists. \
     Install `typescript@^7` or configure `typeChecker.corsaPath`.";

/// Diagnostic source identifiers.
pub mod sources {
    pub const SFC_PARSER: &str = "vize/sfc";
    pub const SFC_COMPILER: &str = "vize/sfc-compile";
    pub const TEMPLATE_PARSER: &str = "vize/template";
    pub const SCRIPT_PARSER: &str = "vize/script";
    pub const JSX_COMPILER: &str = "vize/jsx";
    pub const LINTER: &str = "vize/lint";
    pub const TYPE_CHECKER: &str = "vize/types";
    pub const COMPONENTS: &str = "vize/components";
    pub const MUSEA: &str = "vize/musea";
}
