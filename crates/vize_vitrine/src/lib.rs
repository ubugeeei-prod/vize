//! NAPI and WASM bindings for Vue compiler.
#![cfg_attr(
    feature = "napi",
    expect(
        clippy::disallowed_macros,
        clippy::disallowed_methods,
        reason = "napi-derive expands to std formatting and `to_string`"
    )
)]

#[cfg(any(test, feature = "napi"))]
pub mod napi;

#[cfg(feature = "wasm")]
pub mod wasm;

#[cfg(any(feature = "napi", feature = "wasm"))]
mod parse_errors;
pub mod plugin_output;
#[cfg(any(test, feature = "napi"))]
mod plugin_transform;
#[cfg(any(feature = "napi", feature = "wasm"))]
mod template_syntax;
pub mod typecheck;
pub mod types;
#[cfg(any(feature = "napi", feature = "wasm"))]
mod whitespace;

pub use typecheck::{
    RelatedLocation, TypeCheckOptions, TypeCheckResult, TypeDiagnostic, TypeSeverity,
    type_check_sfc, type_check_sfc_with_legacy_vue2,
};
pub use types::*;
