//! Virtual TypeScript generation for Vue SFC type checking.
//!
//! This module generates TypeScript code that represents a Vue SFC's
//! runtime behavior, enabling type checking of template expressions
//! and script setup bindings.
//!
//! Key design: Uses closures from Croquis scope information instead of
//! `declare const` to properly model Vue's template scoping.

use std::cell::Cell;

#[cfg(test)]
mod dynamic_component_names_tests;
mod expressions;
mod generator;
mod helpers;
pub mod incremental;
#[cfg(test)]
mod interface_extends_tests;
#[cfg(test)]
mod legacy_vue2_vuetify_tests;
mod props;
mod scope;
#[cfg(test)]
mod tests;
mod types;

pub(crate) use generator::generate_virtual_ts_with_offsets_and_checks;
pub use generator::{
    generate_virtual_ts, generate_virtual_ts_with_offsets,
    generate_virtual_ts_with_offsets_legacy_vue2, generate_virtual_ts_with_offsets_options_api,
};
pub use helpers::{DECLARATION_HELPERS_DTS, SHARED_PREAMBLE_DTS, SHARED_PREAMBLE_FILE_NAME};
#[cfg(any(test, feature = "native"))]
pub(crate) use types::VirtualTsCheckOptions;
pub(crate) use types::VirtualTsGenerationOptions;
pub use types::{TemplateGlobal, VirtualTsOptions, VirtualTsOutput, VizeMapping};

thread_local! {
    static AUTHORED_SCRIPT_FALLBACK_PARSE_INVOCATIONS: Cell<u64> = const { Cell::new(0) };
}

#[doc(hidden)]
pub fn authored_script_fallback_parse_invocations() -> u64 {
    AUTHORED_SCRIPT_FALLBACK_PARSE_INVOCATIONS.get()
}

#[doc(hidden)]
pub fn reset_authored_script_fallback_parse_invocations() {
    AUTHORED_SCRIPT_FALLBACK_PARSE_INVOCATIONS.set(0);
}

pub(crate) fn record_authored_script_fallback_parse() {
    AUTHORED_SCRIPT_FALLBACK_PARSE_INVOCATIONS.set(
        AUTHORED_SCRIPT_FALLBACK_PARSE_INVOCATIONS
            .get()
            .saturating_add(1),
    );
}

pub fn generate_virtual_ts_with_offsets_and_lib_references(
    summary: &vize_croquis::Croquis,
    script_content: Option<&str>,
    template_ast: Option<&vize_relief::RootNode<'_>>,
    script_offset: u32,
    template_offset: u32,
    options: &VirtualTsOptions,
    lib_references: &[&str],
) -> VirtualTsOutput {
    generator::generate_virtual_ts_with_offsets_and_checks(
        summary,
        script_content,
        template_ast,
        script_offset,
        template_offset,
        options,
        types::VirtualTsGenerationOptions {
            lib_references: Some(lib_references),
            ..Default::default()
        },
    )
}
