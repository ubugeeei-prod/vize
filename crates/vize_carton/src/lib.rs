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

#[expect(
    clippy::disallowed_types,
    reason = "host profile JSON preserves its existing owned span and counter vectors"
)]
pub mod profile_export;

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
