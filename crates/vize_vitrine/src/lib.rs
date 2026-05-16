//! NAPI and WASM bindings for Vue compiler.
#![cfg_attr(
    feature = "napi",
    allow(clippy::disallowed_macros, clippy::disallowed_methods)
)]

#[cfg(feature = "napi")]
pub mod napi;

#[cfg(feature = "wasm")]
pub mod wasm;

pub mod typecheck;
pub mod types;

pub use typecheck::{
    RelatedLocation, TypeCheckOptions, TypeCheckResult, TypeDiagnostic, TypeSeverity,
    type_check_sfc,
};
pub use types::*;
