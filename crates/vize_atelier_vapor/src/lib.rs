//! Vue Vapor mode compiler.
//!
//! Vapor mode is a new compilation strategy that generates more efficient code
//! by eliminating the virtual DOM overhead for static parts of the template.

#![allow(clippy::collapsible_match)]

pub mod generate;
pub mod generators;
pub mod ir;
pub mod transform;
pub mod transforms;

pub use generate::{generate_vapor, VaporGenerateResult};
pub use generators::{
    build_text_expression, can_inline_text, can_optimize_for, can_use_ternary,
    capitalize_event_name, escape_template, generate_async_component, generate_attribute,
    generate_block, generate_class_binding, generate_component_prop, generate_create_component,
    generate_create_text_node, generate_delegate_event, generate_directive,
    generate_directive_array, generate_dynamic_component, generate_dynamic_slot_name,
    generate_effect_wrapper, generate_event_options, generate_for, generate_for_memo, generate_if,
    generate_if_expression, generate_inline_handler, generate_keep_alive, generate_normalize_slots,
    generate_resolve_component, generate_resolve_directive, generate_scoped_slots,
    generate_set_dynamic_props, generate_set_event, generate_set_prop, generate_set_text,
    generate_slot_function, generate_slot_outlet, generate_style_binding, generate_suspense,
    generate_template_declaration, generate_template_instantiation, generate_text_content,
    generate_to_display_string, generate_v_cloak_removal, generate_v_show,
    generate_with_directives, is_dynamic_slot_name, is_v_pre_element, normalize_prop_key,
    GenerateContext,
};
pub use ir::{
    BlockIRNode, ComponentKind, CreateComponentIRNode, DirectiveIRNode, DynamicFlag,
    EventModifiers, EventOptions, ForIRNode, GetTextChildIRNode, IRDynamicInfo, IREffect,
    IRNodeType, IRProp, IRSlot, IfIRNode, InsertNodeIRNode, NegativeBranch, OperationNode,
    PrependNodeIRNode, RootIRNode, SetDynamicPropsIRNode, SetEventIRNode, SetHtmlIRNode,
    SetPropIRNode, SetTemplateRefIRNode, SetTextIRNode, SlotOutletIRNode,
};
pub use transform::transform_to_ir;
pub use transforms::{
    collect_component_slots, generate_element_template, generate_event_handler,
    generate_model_handler, generate_text_expression, generate_v_show_effect, get_model_arg,
    get_model_event, get_model_modifiers, get_model_value, get_show_condition, get_tag_name,
    has_dynamic_bindings, has_event_listeners, has_lazy_modifier, has_number_modifier,
    has_trim_modifier, is_component, is_dynamic_binding, is_slot_outlet, is_static_element,
    is_template_wrapper, needs_transition, parse_for_alias, should_merge_text_nodes,
    transform_for_node, transform_if_branches, transform_interpolation, transform_slot_outlet,
    transform_text, transform_v_bind, transform_v_bind_dynamic, transform_v_for, transform_v_if,
    transform_v_model, transform_v_on, transform_v_show,
};

use vize_atelier_core::{
    options::{ParserOptions, TransformOptions},
    parser::parse_with_options,
    transform::transform,
};
use vize_carton::{Bump, String};

/// Vapor compiler options
#[derive(Debug, Clone, Default)]
pub struct VaporCompilerOptions {
    /// Whether to prefix identifiers
    pub prefix_identifiers: bool,
    /// Whether in SSR mode
    pub ssr: bool,
    /// Binding metadata
    pub binding_metadata: Option<vize_atelier_core::options::BindingMetadata>,
    /// Whether to inline
    pub inline: bool,
}

/// Vapor compilation result
#[derive(Debug)]
pub struct VaporCompileResult {
    /// Generated code
    pub code: String,
    /// Template strings for static parts
    pub templates: Vec<String>,
    /// Error messages during compilation
    pub error_messages: Vec<String>,
}

/// Compile a Vue template to Vapor mode
pub fn compile_vapor<'a>(
    allocator: &'a Bump,
    source: &'a str,
    options: VaporCompilerOptions,
) -> VaporCompileResult {
    // Parse
    let parser_opts = ParserOptions::default();
    let (mut root, errors) = parse_with_options(allocator, source, parser_opts);

    if !errors.is_empty() {
        return VaporCompileResult {
            code: String::default(),
            templates: Vec::new(),
            error_messages: errors.iter().map(|e| e.message.clone()).collect(),
        };
    }

    // Transform to Vapor IR
    let transform_opts = TransformOptions {
        prefix_identifiers: options.prefix_identifiers,
        ssr: options.ssr,
        binding_metadata: options.binding_metadata,
        inline: options.inline,
        vapor: true,
        ..Default::default()
    };
    transform(allocator, &mut root, transform_opts, None);

    // Transform to Vapor IR
    let ir = transform_to_ir(allocator, &root);

    // Generate Vapor code
    let result = generate_vapor(&ir);

    VaporCompileResult {
        code: result.code,
        templates: result.templates,
        error_messages: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::compile_vapor;
    use vize_carton::Bump;

    fn normalize_code(code: &str) -> String {
        code.lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn test_compile_simple_element() {
        let allocator = Bump::new();
        let result = compile_vapor(&allocator, "<div>hello</div>", Default::default());

        assert!(result.error_messages.is_empty(), "Expected no errors");

        let code = normalize_code(&result.code);

        // Check import statement
        assert!(code.starts_with("import {"), "Should start with import");
        assert!(
            code.contains("template as _template"),
            "Should import template"
        );
        assert!(code.contains("from 'vue'"), "Should import from vue");

        // Check template declaration
        assert!(
            code.contains("const t0 = _template(\"<div>hello</div>\", true)"),
            "Should declare template with full element: {}",
            code
        );

        // Check function structure
        assert!(
            code.contains("export function render(_ctx)"),
            "Should export render function"
        );
        assert!(
            code.contains("const n0 = t0()"),
            "Should instantiate template"
        );
        assert!(code.contains("return n0"), "Should return element");
    }

    #[test]
    fn test_compile_interpolation() {
        let allocator = Bump::new();
        let result = compile_vapor(&allocator, "<div>{{ msg }}</div>", Default::default());

        assert!(result.error_messages.is_empty(), "Expected no errors");

        let code = normalize_code(&result.code);

        // Check imports include renderEffect and setText
        assert!(
            code.contains("renderEffect as _renderEffect"),
            "Should import renderEffect: {}",
            code
        );
        assert!(
            code.contains("setText as _setText"),
            "Should import setText"
        );

        // Check effect for reactive text (single-line format)
        assert!(
            code.contains("_renderEffect(() =>"),
            "Should have render effect: {}",
            code
        );
        assert!(code.contains("_setText("), "Should set text inside effect");
        assert!(code.contains("msg"), "Should reference msg variable");
    }

    #[test]
    fn test_compile_event() {
        let allocator = Bump::new();
        let result = compile_vapor(
            &allocator,
            r#"<button @click="handleClick">Click</button>"#,
            Default::default(),
        );

        assert!(result.error_messages.is_empty(), "Expected no errors");

        let code = normalize_code(&result.code);

        // Check imports
        assert!(
            code.contains("createInvoker as _createInvoker"),
            "Should import createInvoker helper: {}",
            code
        );

        // Check template
        assert!(
            code.contains("_template(\"<button>Click</button>\", true)"),
            "Should have button template: {}",
            code
        );

        // Check event binding
        assert!(
            code.contains("$evtclick = _createInvoker"),
            "Should bind click event with invoker: {}",
            code
        );
    }

    #[test]
    fn test_compile_v_if() {
        let allocator = Bump::new();
        let result = compile_vapor(
            &allocator,
            r#"<div v-if="show">visible</div>"#,
            Default::default(),
        );

        assert!(
            result.error_messages.is_empty(),
            "Expected no errors: {:?}",
            result.error_messages
        );

        let code = normalize_code(&result.code);

        // v-if should generate createIf
        assert!(
            code.contains("_createIf"),
            "Should use createIf for v-if: {}",
            code
        );
        assert!(code.contains("show"), "Should reference show condition");
    }

    #[test]
    fn test_compile_v_for() {
        let allocator = Bump::new();
        let result = compile_vapor(
            &allocator,
            r#"<div v-for="item in items">{{ item }}</div>"#,
            Default::default(),
        );

        assert!(
            result.error_messages.is_empty(),
            "Expected no errors: {:?}",
            result.error_messages
        );

        let code = normalize_code(&result.code);

        // v-for should generate createFor
        assert!(
            code.contains("_createFor"),
            "Should use createFor for v-for: {}",
            code
        );
        assert!(code.contains("items"), "Should reference items source");
    }
}
