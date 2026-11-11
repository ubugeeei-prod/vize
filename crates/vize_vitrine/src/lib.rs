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

/// The addon runs the content mapper, `vize check` and the rest of the CLI
/// inside node (`runCli`), so it installs the CLI binary's allocator instead
/// of the platform malloc. macOS keeps the system allocator (see Cargo.toml).
#[cfg(all(feature = "napi", any(target_os = "linux", windows)))]
mod allocator {
    #[global_allocator]
    static GLOBAL_ALLOCATOR: mimalloc::MiMalloc = mimalloc::MiMalloc;

    vize::commit_mimalloc_arenas_on_demand!();
}

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
