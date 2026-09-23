//! Diagnostics aggregation from multiple sources.
//!
//! Aggregates diagnostics from:
//! - SFC parser errors
//! - Template parser errors
//! - vize_patina (linter)
//! - Future: vize_canon (type checker)
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    reason = "tower-lsp lsp_types take std String/HashMap values, built with to_string/format!"
)]

#[cfg(all(test, feature = "native"))]
mod art_dependency_typecheck_tests;
#[cfg(all(test, feature = "native"))]
mod art_variant_typecheck_tests;
#[cfg(all(test, feature = "native"))]
mod assembly_parity_tests;
mod builder;
mod cached_descriptor;
mod collectors;
mod component_props;
#[cfg(test)]
mod component_props_tests;
#[cfg(test)]
mod configured_lint_tests;
#[cfg(feature = "native")]
pub(in crate::ide) mod corsa;
#[cfg(all(test, feature = "native"))]
mod editor_typecheck_fixture;
#[cfg(all(test, feature = "native"))]
mod editor_typecheck_tests;
mod line_index;
mod linter_options;
#[cfg(feature = "native")]
mod native;
mod service;
mod severity;
#[expect(
    clippy::disallowed_macros,
    reason = "insta snapshot macros expand through std::format!; see CONTRIBUTING.md, \"Snapshot assertions in test targets\""
)]
#[cfg(test)]
mod tests;
#[cfg(all(test, feature = "native"))]
mod typecheck_unavailable_tests;
mod vize_sfc_type;
#[cfg(all(test, feature = "native"))]
mod vize_sfc_type_tests;

pub use builder::DiagnosticBuilder;
pub use service::DiagnosticService;
pub use severity::Severity;

pub(in crate::ide) use line_index::LineIndex;
#[cfg(test)]
pub(in crate::ide) use line_index::offset_to_line_col;
#[cfg(feature = "native")]
pub(in crate::ide) use service::VirtualTsResult;

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
