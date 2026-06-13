//! Vue Vapor mode compiler.
//!
//! Vapor mode is a new compilation strategy that generates more efficient code
//! by eliminating the virtual DOM overhead for static parts of the template.

#![allow(clippy::collapsible_match)]

pub mod compile;
pub mod generate;
pub mod generators;
pub mod ir;
pub mod lower;
pub mod steps;

#[cfg(test)]
mod tests;

pub use compile::{
    VaporCompileResult, VaporCompilerOptions, compile_vapor, compile_vapor_with_diagnostics,
    compile_vapor_with_template_syntax, compile_vapor_with_template_syntax_and_diagnostics,
};
#[allow(deprecated)]
pub use compile::{
    compile_vapor_with_vue_parser_quirks, compile_vapor_with_vue_parser_quirks_and_diagnostics,
};
pub use generate::{
    VaporGenerateOptions, VaporGenerateResult, generate_vapor, generate_vapor_with_options,
};
pub use generators::{
    GenerateContext, build_text_expression, can_inline_text, can_optimize_for, can_use_ternary,
    capitalize_event_name, escape_template, generate_async_component, generate_attribute,
    generate_block, generate_class_binding, generate_component_prop, generate_create_component,
    generate_create_text_node, generate_delegate_event, generate_directive,
    generate_directive_array, generate_dynamic_component, generate_effect_wrapper,
    generate_event_options, generate_for, generate_for_memo, generate_if, generate_if_expression,
    generate_inline_handler, generate_keep_alive, generate_resolve_component,
    generate_resolve_directive, generate_set_dynamic_props, generate_set_event, generate_set_prop,
    generate_set_text, generate_style_binding, generate_suspense, generate_template_declaration,
    generate_template_instantiation, generate_text_content, generate_to_display_string,
    generate_v_cloak_removal, generate_v_show, generate_with_directives, is_v_pre_element,
    normalize_prop_key,
};
pub use ir::{
    BlockIRNode, ComponentKind, CreateComponentIRNode, DirectiveIRNode, DynamicFlag,
    EventModifiers, EventOptions, ForIRNode, GetTextChildIRNode, IRDynamicInfo, IREffect,
    IRNodeType, IRProp, IRSlot, IfIRNode, InsertNodeIRNode, NegativeBranch, OperationNode,
    PrependNodeIRNode, RootIRNode, SetDynamicPropsIRNode, SetEventIRNode, SetHtmlIRNode,
    SetPropIRNode, SetTemplateRefIRNode, SetTextIRNode, SlotOutletIRNode,
};
pub use lower::transform_to_ir;
pub use steps::{
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
