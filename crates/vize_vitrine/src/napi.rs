//! NAPI bindings for Vue compiler.
//!
//! This module is organized into:
//! - `template`: Template compilation (compile, compileVapor, parseTemplate)
//! - `sfc`: SFC parsing, compilation, and batch processing
//! - `art`: Art file parsing, CSF transform, docs, palette, and autogen
//! - `format`: Vue SFC formatting
//! - `lint`: Vue SFC linting
//! - `config`: Public config normalization helpers

#[cfg(feature = "napi")]
mod art;
#[cfg(feature = "napi")]
mod art_status;
#[cfg(feature = "napi")]
mod build_token;
#[cfg(feature = "napi")]
mod cli;
#[cfg(feature = "napi")]
mod config;
#[cfg(feature = "napi")]
mod format;
#[cfg(feature = "napi")]
mod inspector;
#[cfg(feature = "napi")]
mod jsx;
#[cfg(feature = "napi")]
mod lint;
mod lint_fix;
#[cfg(feature = "napi")]
mod plugin;
#[cfg(feature = "napi")]
mod plugin_output;
#[cfg(feature = "napi")]
mod plugin_sdk;
#[cfg(feature = "napi")]
mod sfc;
#[cfg(feature = "napi")]
mod template;
#[cfg(feature = "napi")]
mod tokens;
#[cfg(feature = "napi")]
mod transform_plugins;

#[cfg(feature = "napi")]
mod napi_typecheck;
#[cfg(feature = "napi")]
pub use napi_typecheck::*;

// The same module names and files exercise the pure implementation without
// linking Node's N-API. Each feature configuration discovers every test once.
#[cfg(all(test, not(feature = "napi")))]
#[expect(
    dead_code,
    reason = "the N-API callers are not linked in this test configuration"
)]
mod lint {
    mod file_collection {
        pub(crate) mod oxlint_html_profile;
    }
    #[expect(
        clippy::disallowed_types,
        reason = "the unchanged FFI DTOs own JavaScript strings"
    )]
    mod lint_options;
    mod oxlint_html;
    mod rule_metadata;
}

#[cfg(all(test, not(feature = "napi")))]
#[expect(
    dead_code,
    reason = "the N-API callers are not linked in this test configuration"
)]
mod plugin_sdk {
    mod batch;
    mod document;
    mod error;
    mod facts;
    mod plugin_cache;
    mod production;
    mod providers;
    mod tests;
}

#[cfg(all(test, not(feature = "napi")))]
mod sfc {
    mod analysis;
}

#[cfg(feature = "napi")]
pub use art::*;
#[cfg(feature = "napi")]
pub use art_status::*;
#[cfg(feature = "napi")]
pub use build_token::*;
#[cfg(feature = "napi")]
pub use cli::*;
#[cfg(feature = "napi")]
pub use config::*;
#[cfg(feature = "napi")]
pub use format::*;
#[cfg(feature = "napi")]
pub use inspector::*;
#[cfg(feature = "napi")]
pub use jsx::*;
#[cfg(feature = "napi")]
pub use lint::*;
#[cfg(feature = "napi")]
pub use plugin::*;
#[cfg(feature = "napi")]
pub use plugin_output::*;
#[cfg(feature = "napi")]
pub use plugin_sdk::*;
#[cfg(feature = "napi")]
pub use sfc::*;
#[cfg(feature = "napi")]
pub use template::*;
#[cfg(feature = "napi")]
pub use tokens::*;
#[cfg(feature = "napi")]
pub use transform_plugins::*;
