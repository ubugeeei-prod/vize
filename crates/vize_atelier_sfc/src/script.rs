//! Script block analysis and compilation.
//!
//! This module handles the compilation of `<script>` and `<script setup>` blocks,
//! including Compiler Macros like `defineProps`, `defineEmits`, etc.
//!
//! Module structure follows Vue.js official implementation.
//! Uses OXC for JavaScript/TypeScript parsing instead of Babel.

mod analyze_script_bindings;
mod context;
mod define_emits;
mod define_expose;
mod define_model;
mod define_model_metadata;
mod define_options;
mod define_props;
mod define_props_destructure;
mod define_slots;
mod import_usage_check;
mod static_expression;
#[cfg(test)]
mod tsx_setup_tests;
pub(crate) mod type_resolution;
mod utils;

// Re-export main types
pub(crate) use analyze_script_bindings::get_object_or_array_argument_keys;
pub use analyze_script_bindings::{analyze_script_bindings, get_object_or_array_expression_keys};
pub use context::{
    ScriptCompileContext, SfcTypeProject, TypeResolutionBatchGuard, TypeSourceSnapshot,
    begin_type_resolution_batch, join_type_resolution_batch,
};
pub use define_emits::{
    DefineEmitsResult, extract_runtime_emits, gen_runtime_emits, process_define_emits,
};
pub(crate) use define_model_metadata::{
    define_model_metadata, define_model_name, define_model_prop_option_spans,
};
pub use define_props_destructure::{
    PropsDestructureBinding, PropsDestructuredBindings, gen_props_access_exp,
    process_props_destructure, transform_destructured_props,
};
pub(crate) use define_props_destructure::{
    transform_define_expose_destructured_props, transform_destructured_props_with_edits,
};
pub use import_usage_check::{
    TemplateUsedIdentifiers, is_used_in_template, resolve_template_read_identifiers,
    resolve_template_used_identifiers, resolve_template_v_model_identifiers,
};
pub(crate) use static_expression::{hoistable_literal_name, is_static_enum, register_enum};
pub(crate) use type_resolution::{
    build_interface_type_source, resolve_type_args, resolve_type_to_object_body,
};
pub(crate) use utils::model_modifiers_binding_name;
pub use utils::{
    MacroCall, ScriptSetupMacros, get_escaped_prop_name, is_compiler_macro_line,
    is_valid_identifier,
};

// Re-export constants
pub use define_emits::DEFINE_EMITS;
pub use define_expose::DEFINE_EXPOSE;
pub use define_model::DEFINE_MODEL;
pub use define_options::DEFINE_OPTIONS;
pub use define_props::{DEFINE_PROPS, WITH_DEFAULTS};
pub use define_slots::DEFINE_SLOTS;

use crate::types::BindingMetadata;

/// Analyze script setup and extract bindings
pub fn analyze_script_setup(content: &str) -> BindingMetadata {
    let mut ctx = ScriptCompileContext::new(content);
    ctx.analyze();
    ctx.bindings
}

/// Extract macro calls from script setup
pub fn extract_macros(content: &str) -> ScriptSetupMacros {
    let mut ctx = ScriptCompileContext::new(content);
    ctx.extract_all_macros();
    ctx.macros
}

/// Parse script text with `source_type`. A `lang="tsx"` or `"jsx"` block may
/// carry JSX, which the TypeScript grammar rejects; text that only parses with
/// JSX enabled is read that way, so stages without the authored `lang` agree
/// with the TSX setup program.
pub(crate) fn parse_script_text<'a>(
    allocator: &'a oxc_allocator::Allocator,
    source: &'a str,
    source_type: oxc_span::SourceType,
) -> oxc_parser::ParserReturn<'a> {
    let parsed = oxc_parser::Parser::new(allocator, source, source_type).parse();
    if (parsed.panicked || !parsed.diagnostics.is_empty())
        && !source_type.is_jsx()
        && source.contains('<')
    {
        let jsx = oxc_parser::Parser::new(allocator, source, source_type.with_jsx(true)).parse();
        if !jsx.panicked && jsx.diagnostics.is_empty() {
            return jsx;
        }
    }
    parsed
}
