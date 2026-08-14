//! NAPI and WASM bindings for Vue compiler.
#![cfg_attr(
    feature = "napi",
    allow(clippy::disallowed_macros, clippy::disallowed_methods)
)]

#[cfg(feature = "napi")]
pub mod napi;

// Keep the pure lint/fix path covered without linking a standalone test binary
// against Node's N-API symbols.
#[cfg(all(test, not(feature = "napi")))]
#[path = "napi/lint_fix.rs"]
mod lint_fix_tests;

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
