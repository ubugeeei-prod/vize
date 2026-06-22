//! Virtual TypeScript generation for Vue SFC type checking.
//!
//! This module generates TypeScript code that represents a Vue SFC's
//! runtime behavior, enabling type checking of template expressions
//! and script setup bindings.
//!
//! Key design: Uses closures from Croquis scope information instead of
//! `declare const` to properly model Vue's template scoping.

mod expressions;
mod generator;
mod helpers;
pub mod incremental;
#[cfg(test)]
mod legacy_vue2_vuetify_tests;
mod props;
mod scope;
#[cfg(test)]
mod tests;
mod types;

#[cfg(any(test, feature = "native"))]
pub(crate) use generator::generate_virtual_ts_with_offsets_and_checks;
pub use generator::{
    generate_virtual_ts, generate_virtual_ts_with_offsets,
    generate_virtual_ts_with_offsets_legacy_vue2, generate_virtual_ts_with_offsets_options_api,
};
pub use helpers::{DECLARATION_HELPERS_DTS, SHARED_PREAMBLE_DTS, SHARED_PREAMBLE_FILE_NAME};
pub use types::{TemplateGlobal, VirtualTsOptions, VirtualTsOutput, VizeMapping};
#[cfg(any(test, feature = "native"))]
pub(crate) use types::{VirtualTsCheckOptions, VirtualTsGenerationOptions};
