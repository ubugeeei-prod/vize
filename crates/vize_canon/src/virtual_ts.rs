//! Virtual TypeScript generation for Vue SFC type checking.
//!
//! This module generates TypeScript code that represents a Vue SFC's
//! runtime behavior, enabling type checking of template expressions
//! and script setup bindings.
//!
//! Key design: Uses closures from Croquis scope information instead of
//! `declare const` to properly model Vue's template scoping.

#[cfg(test)]
mod class_component_props_tests;
mod component_aliases;
mod component_reference;
mod component_tag;
#[cfg(test)]
mod defaults_key_links_tests;
#[cfg(test)]
mod define_emits_usage_tests;
pub(crate) mod diagnostic_ownership;
#[cfg(test)]
mod dynamic_component_names_tests;
#[cfg(test)]
mod dynamic_component_scope_tests;
#[cfg(test)]
mod event_handler_tests;
mod expressions;
pub(crate) mod generated_names;
#[cfg(test)]
mod generated_reference_scope_tests;
pub(crate) use expressions::NAMED_PROP_DIAGNOSTIC_MARKER;
#[cfg(feature = "native")]
pub(crate) use expressions::key_sub_spans;
pub(crate) use expressions::unchecked_prop_values::{ATTR_VALUE_REFERENCE, attr_value_declaration};
mod generator;
mod reference_paths;
#[cfg(feature = "native")]
pub(crate) use generator::generic_recovery;
pub(crate) use reference_paths::validate_reference_paths;
pub(crate) mod generic_parameters;
pub use generic_parameters::{GenericParameters, authored_generic_range};
mod helpers;
pub use helpers::shared_types::SHARED_VIRTUAL_TYPE_EXPORTS;
mod import_meta;
#[cfg(feature = "native")]
mod instantiated_component_prop;
#[cfg(feature = "native")]
pub use instantiated_component_prop::instantiated_component_prop_key_offset;
pub mod incremental;
#[cfg(test)]
mod interface_extends_tests;
#[cfg(test)]
mod legacy_vue2_vuetify_tests;
#[cfg(test)]
mod loose_is_listener_tests;
mod macro_type_mappings;
pub mod mapping;
mod model_types;
#[cfg(test)]
mod native_attr_name_tests;
mod pattern_diagnostics;
mod props;
#[cfg(test)]
mod public_instance_guard_tests;
mod scope;
mod script_facts;
mod semantic_links;
#[cfg(test)]
mod shared_jsx_helpers_tests;
#[cfg(test)]
mod strict_template_global_enum_tests;
#[cfg(test)]
mod strict_template_global_fallback_tests;
#[expect(clippy::disallowed_macros, reason = "fixtures use std strings")]
#[cfg(test)]
mod strict_template_globals_tests;
#[cfg(test)]
mod strict_template_scope_tests;
mod template_binding_access;
#[expect(clippy::string_slice, reason = "tests assert by panicking")]
#[expect(clippy::disallowed_macros, reason = "fixtures use std strings")]
#[expect(clippy::disallowed_methods, reason = "fixtures use std strings")]
#[expect(clippy::disallowed_types, reason = "fixtures use std strings")]
#[cfg(test)]
mod tests;
mod text_buf;
mod type_dependencies;
pub(crate) use text_buf::{TextBuf, TsText};
mod types;
#[cfg(test)]
mod unknown_props_tests;

pub use generator::setup_helpers::owned_emit_navigation_source_ranges;
#[cfg(feature = "original-import-products")]
pub use generator::{
    OriginalImportError, OriginalImportFacts, entry::generate_virtual_ts_with_original_imports,
};
pub use generator::{
    entry::generate_virtual_ts_with_generic, entry::generate_virtual_ts_with_split_offsets,
    generate_virtual_ts, generate_virtual_ts_with_offsets,
    generate_virtual_ts_with_offsets_legacy_vue2, generate_virtual_ts_with_offsets_options_api,
};
pub(crate) use generator::{
    extract_declared_name, generate_virtual_ts_with_offsets_and_checks,
    generate_virtual_ts_with_shared_parses,
};
#[cfg(feature = "native")]
pub(crate) use helpers::ERROR_COMPONENT_STUB;
pub use helpers::{
    DECLARATION_HELPERS_DTS, SHARED_PREAMBLE_DTS, SHARED_PREAMBLE_FILE_NAME,
    SHARED_PREAMBLE_STRICT_DTS, VUE_SETUP_HELPERS, VUE_TYPE_HELPERS,
};
#[cfg(feature = "native")]
pub(crate) use helpers::{push_ts_string_literal, to_safe_identifier};
pub use mapping::{
    ProjectionFeatures, ProjectionMapping, ProjectionMeta, ProjectionRow, ProjectionSpanKind,
    VizeMapping, VizeSubSpan, virtual_ts_document,
};
pub use pattern_diagnostics::is_unreachable_pattern_diagnostic;
pub use semantic_links::{VizeSemanticLink, VizeSemanticLinkKind};
pub(crate) use types::RESOLVED_RUNTIME_DEFAULT;
#[cfg(feature = "native")]
pub(crate) use types::{CSS_MODULE_GLOBAL_MARKER, TYPED_ROUTE_GLOBAL_MARKER};
pub use types::{TemplateGlobal, TemplateRefOwnership, VirtualTsOptions, VirtualTsOutput};

/// Shared type-only component contract for plain-TS JSX lowering in batch and
/// editor paths. Keep one declaration source so the two consumers cannot drift.
///
/// `$JsxFallthroughAttrs` admits `class`/`style`, `data-`/`aria-`
/// attributes, and native DOM listeners not shadowed by a declared prop. The
/// listener set is a **deliberate divergence from `vue-tsc`**, which rejects
/// `<Comp onClick={…}/>` outright for a component that does not declare it even
/// though Vue forwards the listener to the fallthrough root at runtime; vize
/// accepts it *and* contextually types the event payload, so
/// `onClick={(event: string) => …}` is still an error. See
/// `crates/vize/tests/check_jsx_component_contract_cli.rs`, which pins both
/// halves of that contract.
///
/// `$JsxSlotPayload` is the JSX analogue of the `.vue` template path's
/// `slot_props_type` (`virtual_ts::scope::emit`): a scoped slot's parameter is
/// typed from the host component's declared `$slots`, falling back to `any`
/// whenever the host or the slot is untyped so untyped slot hosts never produce
/// a false positive.
///
/// It resolves `$slots` on *constructable* hosts only, which covers the imported
/// `.vue` components #4042 is about. A local function component declares its
/// slots on the second `Ctx<Emits, Slots>` parameter instead, and those payloads
/// are intentionally left `any` here: that fallback is the conservative side of
/// the trade (it can miss an error, never fabricate one), and typing them is
/// tracked separately from this fix.
pub const JSX_COMPONENT_HELPER: &str =
    include_str!("virtual_ts/helpers/declarations/jsx-components.d.ts.txt");
#[cfg(feature = "native")]
pub(crate) use types::AuthoredScriptMetadata;
#[cfg(feature = "native")]
pub(crate) use types::ResolveStyleClassNames;
#[cfg(any(test, feature = "native"))]
pub(crate) use types::VirtualTsCheckOptions;
pub(crate) use types::{ComponentHelperModule, VirtualTsGenerationOptions};

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

#[cfg(feature = "plain-script-bridges")]
pub use generator::entry::{PlainScriptBridge, PlainScriptBridgeError, PlainScriptExportText};
#[cfg(feature = "original-style-bindings")]
mod style_bindings;
#[cfg(feature = "original-style-bindings")]
pub use style_bindings::{StyleBindingInputError, StyleBindingReferences};
