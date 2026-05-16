//! Transform plugins for Vue template AST.
//!
//! This module contains individual transform plugins that process specific
//! directives and node types during the transform phase.

pub mod hoist_static;
pub mod transform_element;
pub mod transform_expression;
pub mod transform_text;
pub mod v_bind;
pub mod v_for;
pub mod v_if;
pub mod v_memo;
pub mod v_model;
pub mod v_on;
pub mod v_once;
pub mod v_slot;

pub use hoist_static::{
    count_dynamic_children, get_static_type, hoist_static, is_static_node, should_use_block,
    StaticType,
};
pub use transform_element::{
    build_element_codegen, build_props, resolve_element_type, ChildrenType, PropItem,
    TransformPropsExpression, TransformVNodeCall,
};
pub use transform_expression::{
    is_event_handler_reference_expression, is_simple_identifier, prefix_identifiers_in_expression,
    process_expression, process_inline_handler, strip_typescript_from_expression,
};
pub use transform_text::{
    build_text_call, condense_whitespace, is_condensible_whitespace, is_whitespace_only,
    transform_text_children, TextCallExpression, TextPart,
};
pub use v_bind::{
    camelize, get_bind_name, get_bind_value, has_attr_modifier, has_camel_modifier,
    has_prop_modifier, is_dynamic_binding, process_v_bind,
};
pub use v_for::{
    get_for_expression, has_v_for, parse_for_expression, process_v_for, remove_for_directive,
};
pub use v_if::{
    get_if_condition, has_v_else, has_v_else_if, has_v_if, process_v_if, remove_if_directive,
};
pub use v_memo::{
    generate_memo_check, generate_v_memo_wrapper, get_memo_deps, get_memo_exp, has_v_memo,
    process_v_memo, remove_v_memo, MemoInfo,
};
pub use v_model::{
    get_model_event_prop, get_vmodel_helper, parse_model_modifiers, supports_v_model,
    transform_v_model, VModelModifiers,
};
pub use v_on::{
    create_on_name, get_event_name, get_handler_expression, is_dynamic_event, needs_guard,
    parse_event_modifiers, process_v_on, EventModifiers,
};
pub use v_once::{generate_v_once_wrapper, has_v_once, remove_v_once, transform_v_once};
pub use v_slot::{
    collect_slots, get_slot_name, get_slot_props_string, has_dynamic_slots, has_v_slot,
    is_dynamic_slot, transform_slot_outlet, SlotInfo, SlotOutletInfo,
};
