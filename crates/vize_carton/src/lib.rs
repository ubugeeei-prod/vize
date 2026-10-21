//! Legacy storage compatibility and host-runtime integration.
//! Portable storage identities are re-exported from `vize_l0`.
pub use vize_l0::*;

pub mod config;
#[expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    reason = "host translation parsing retains its existing std string and unknown-key contract"
)]
pub mod i18n;
mod i18n_compiler;
mod i18n_compiler_directive;
mod i18n_compiler_template;
mod i18n_croquis;
mod i18n_croquis_last;
mod i18n_croquis_more;
mod i18n_croquis_rest;
mod i18n_explain;
mod i18n_l3;
mod i18n_render;
mod i18n_rules_ecosystem;
mod i18n_rules_markup;
mod i18n_rules_script;
mod i18n_rules_script_more;
mod i18n_supplemental;
mod i18n_supplemental_extra;
mod i18n_supplemental_extra2;
mod i18n_supplemental_html;

pub mod profile_allocator;
pub mod profile_export;

pub mod source_io;

#[cfg(not(target_arch = "wasm32"))]
pub mod corsa_api_mode;
#[cfg(not(target_arch = "wasm32"))]
#[expect(
    clippy::disallowed_types,
    reason = "legacy host discovery retains its environment and path collections"
)]
pub mod corsa_resolver;

#[expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    reason = "host JSON-RPC records retain the existing std String and Vec contract"
)]
pub mod lsp;

pub mod path;

pub mod timing_observer;

/// The Vize version users see (`vize --version`, LSP server info).
///
/// Release builds receive the exact planned version explicitly so Cargo tracks
/// it even when trusted target caches were populated by a different release.
/// Legacy release callers fall back to their stamped Cargo package version;
/// every other build reports `<version>-dev`.
pub const VERSION: &str = match option_env!("VIZE_RELEASE_VERSION") {
    Some(version) => version,
    None => match option_env!("VIZE_RELEASE_BUILD") {
        Some(_) => env!("CARGO_PKG_VERSION"),
        None => concat!(env!("CARGO_PKG_VERSION"), "-dev"),
    },
};

#[cfg(test)]
mod version_tests {
    #[test]
    fn unreleased_builds_identify_as_dev() {
        match option_env!("VIZE_RELEASE_VERSION") {
            Some(version) => assert_eq!(super::VERSION, version),
            None => match option_env!("VIZE_RELEASE_BUILD") {
                Some(_) => assert_eq!(super::VERSION, env!("CARGO_PKG_VERSION")),
                None => assert_eq!(super::VERSION, concat!(env!("CARGO_PKG_VERSION"), "-dev")),
            },
        }
    }
}
