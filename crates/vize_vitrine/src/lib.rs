//! NAPI and WASM bindings for Vue compiler.
#![cfg_attr(
    feature = "napi",
    expect(
        clippy::disallowed_macros,
        clippy::disallowed_methods,
        reason = "napi-derive expands to std formatting and `to_string`"
    )
)]

#[cfg(feature = "napi")]
pub mod napi;

// Keep the pure lint/fix path covered without linking a standalone test binary
// against Node's N-API symbols.
#[cfg(test)]
#[cfg(not(feature = "napi"))]
#[path = "napi/lint_fix.rs"]
mod lint_fix_tests;

// The P4-16 JS plugin host's pure half (document, facts, batch), tested the
// same way.
#[cfg(test)]
#[cfg(not(feature = "napi"))]
#[path = "napi/plugin_sdk"]
#[expect(
    dead_code,
    reason = "the napi half that reads the rest is not compiled here"
)]
mod plugin_sdk_host {
    mod batch;
    mod document;
    mod error;
    mod facts;
    mod plugin_cache;
    mod tests;
}

#[cfg(feature = "wasm")]
pub mod wasm;

#[cfg(any(feature = "napi", feature = "wasm"))]
mod parse_errors;
#[cfg(any(feature = "napi", feature = "wasm"))]
mod template_syntax;
pub mod typecheck;
pub mod types;

pub use typecheck::{
    RelatedLocation, TypeCheckOptions, TypeCheckResult, TypeDiagnostic, TypeSeverity,
    type_check_sfc, type_check_sfc_with_legacy_vue2,
};
pub use types::*;
