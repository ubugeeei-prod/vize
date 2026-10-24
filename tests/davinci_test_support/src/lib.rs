//! Shared Davinci test infrastructure.
//!
//! Keeping cross-crate fixtures and validators in a regular dependency avoids
//! relative `#[path]` includes while preserving one source of truth.

#[expect(
    clippy::panic,
    reason = "test support: the corpus gate fails the calling test with a diagnostic by panicking"
)]
pub mod corpus;
pub mod schema;
pub mod surface_fixture;

#[cfg(feature = "formatter")]
pub mod formatter_continuation_reference;

#[cfg(feature = "formatter")]
pub mod formatter_expression_width_reference;

#[cfg(feature = "reference-path")]
pub mod reference_path_project;

#[cfg(feature = "native-process")]
pub mod original_diagnosing_process;

#[cfg(feature = "native-dom")]
pub mod native_dom_file;

#[cfg(feature = "ssr")]
pub mod ssr_fix_history;

#[cfg(feature = "panic-boundary")]
pub mod css_fuzz_boundary;

#[cfg(feature = "vue-fixtures")]
pub mod unknown_template_fixture;
#[cfg(feature = "vue-fixtures")]
pub mod vue_helper_fixture;

#[cfg(feature = "canon-project")]
pub mod script_block_project;

#[cfg(feature = "formatter")]
pub mod css_multi_value_reference;
