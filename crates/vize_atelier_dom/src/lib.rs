//! Vue compiler for DOM platform.
//!
//! This module provides DOM-specific compilation including:
//! - DOM element and attribute validation
//! - v-model transforms for form elements
//! - v-on event modifiers
//! - v-show transform
//! - Style and class binding handling

#![allow(clippy::collapsible_match)]
#![cfg_attr(
    test,
    allow(clippy::disallowed_macros, clippy::field_reassign_with_default)
)]

mod compile;
mod namespace;
pub mod options;
pub mod transforms;
pub use transforms as passes;

#[cfg(test)]
mod tests;

pub use compile::{
    compile_template, compile_template_with_options,
    compile_template_with_options_and_hoisted_scope_id, compile_template_with_template_syntax,
    compile_template_with_template_syntax_and_hoisted_scope_id,
    compile_template_with_template_syntax_and_hoisted_scope_id_with_sections,
};
#[allow(deprecated)]
pub use compile::{
    compile_template_with_vue_parser_quirks,
    compile_template_with_vue_parser_quirks_and_hoisted_scope_id,
};
pub use options::{DomCompilerOptions, element_checks, event_modifiers};
pub use transforms::{
    EventModifiers, EventOptions, MouseModifiers, PropagationModifiers, SystemModifiers, V_SHOW,
    V_TEXT, VModelModifiers, generate_html_prop, generate_html_warning, generate_key_guard,
    generate_model_props, generate_modifier_guard, generate_show_directive, generate_show_style,
    generate_text_children, generate_text_content, get_model_event, get_model_helper,
    get_model_prop, is_v_html, is_v_show, is_v_text, resolve_key_alias,
};

// Re-export core types
pub use vize_atelier_core::{
    Allocator, CompilerError, ElementNode, Namespace, RootNode, TemplateChildNode, codegen, errors,
    parser, pipeline, runtime_helpers, tokenizer, transform,
};
