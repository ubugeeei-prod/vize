//! Vue compiler for DOM platform.
//!
//! This module provides DOM-specific compilation including:
//! - DOM element and attribute validation
//! - v-model transforms for form elements
//! - v-on event modifiers
//! - v-show transform
//! - Style and class binding handling

#![allow(clippy::collapsible_match)]
#![allow(deprecated)]
#![cfg_attr(
    test,
    allow(clippy::disallowed_macros, clippy::field_reassign_with_default)
)]

#[cfg(feature = "graph")]
mod atlas;
#[cfg(feature = "legacy")]
mod compile;
#[cfg(all(test, feature = "legacy"))]
mod experimental_tests;
#[cfg(feature = "legacy")]
mod namespace;
#[cfg(feature = "legacy")]
pub mod options;
#[cfg(feature = "graph")]
mod rendu;
#[cfg(feature = "legacy")]
pub mod steps;

#[cfg(all(test, feature = "legacy"))]
mod tests;

#[cfg(feature = "graph")]
pub use atlas::{DomOutputArtifact, DomOutputProduct, DomProvider, register_atlas_provider};
#[cfg(feature = "legacy")]
pub use compile::{
    compile_template,
    compile_template_root_with_template_syntax_and_hoisted_scope_id_with_sections,
    compile_template_with_options, compile_template_with_options_and_hoisted_scope_id,
    compile_template_with_template_syntax,
    compile_template_with_template_syntax_and_hoisted_scope_id,
    compile_template_with_template_syntax_and_hoisted_scope_id_with_sections,
};
#[cfg(feature = "legacy")]
#[allow(deprecated)]
pub use compile::{
    compile_template_with_vue_parser_quirks,
    compile_template_with_vue_parser_quirks_and_hoisted_scope_id,
};
#[cfg(feature = "legacy")]
pub use options::{DomCompilerOptions, element_checks, event_modifiers};
#[cfg(feature = "graph")]
pub use rendu::{RenduDomMapping, RenduDomOutput, compile_rendu};
#[cfg(feature = "legacy")]
pub use steps::{
    EventModifiers, EventOptions, MouseModifiers, PropagationModifiers, SystemModifiers, V_SHOW,
    V_TEXT, VModelModifiers, generate_html_prop, generate_html_warning, generate_key_guard,
    generate_model_props, generate_modifier_guard, generate_show_directive, generate_show_style,
    generate_text_children, generate_text_content, get_model_event, get_model_helper,
    get_model_prop, is_v_html, is_v_show, is_v_text, resolve_key_alias,
};

// Preserve the public compatibility surface without routing owned parser,
// syntax, or allocator types through Atelier Core.
#[cfg(feature = "legacy")]
pub use vize_armature::{parser, tokenizer};
#[cfg(feature = "legacy")]
pub use vize_atelier_core::{codegen, lane, runtime_helpers, transform};
#[cfg(feature = "legacy")]
pub use vize_carton::Allocator;
#[cfg(feature = "legacy")]
pub use vize_relief::{CompilerError, ElementNode, Namespace, RootNode, TemplateChildNode, errors};
