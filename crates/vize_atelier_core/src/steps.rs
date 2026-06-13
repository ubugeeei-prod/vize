//! Transform steps for Vue template AST.
//!
//! This module contains individual transform steps that process specific
//! directives and node types inside the template lane.

#[path = "transforms/transform_element.rs"]
pub mod element;
#[path = "transforms/transform_expression.rs"]
pub mod expression;
#[path = "transforms/hoist_static.rs"]
pub mod hoist_static;
/// Legacy Vue (v2 / v2.7) template-sugar pre-transforms (`.sync`, scoped-slot
/// attributes) and v-on event-modifier sugar (`.native`, numeric keycodes).
/// Compiled only with the `legacy` cargo feature; a no-op for the default Vue 3
/// dialect.
#[cfg(feature = "legacy")]
#[path = "transforms/legacy.rs"]
pub mod legacy;
/// Vue 2 pipe-filter parsing/rewriting. Legacy-only and dialect-gated; see the
/// module docs. Compiled only behind the `legacy` cargo feature.
#[cfg(feature = "legacy")]
#[path = "transforms/legacy_filters.rs"]
pub(crate) mod legacy_filters;
#[path = "transforms/transform_text.rs"]
pub mod text;
#[path = "transforms/v_bind.rs"]
pub mod v_bind;
#[path = "transforms/v_for.rs"]
pub mod v_for;
#[path = "transforms/v_if.rs"]
pub mod v_if;
#[path = "transforms/v_memo.rs"]
pub mod v_memo;
#[path = "transforms/v_model.rs"]
pub mod v_model;
#[path = "transforms/v_on.rs"]
pub mod v_on;
#[path = "transforms/v_once.rs"]
pub mod v_once;
#[path = "transforms/v_slot.rs"]
pub mod v_slot;

pub use element::{
    ChildrenType, PropItem, TransformPropsExpression, TransformVNodeCall, build_element_codegen,
    build_props, resolve_element_type,
};
pub use expression::{
    is_event_handler_reference_expression, is_simple_identifier, prefix_identifiers_in_expression,
    process_expression, process_inline_handler, strip_typescript_from_expression,
};
pub use hoist_static::{
    StaticType, count_dynamic_children, get_static_type, hoist_static, is_static_node,
    should_use_block,
};
pub use text::{
    TextCallExpression, TextPart, build_text_call, condense_whitespace, is_condensible_whitespace,
    is_whitespace_only, transform_text_children,
};
pub use v_bind::{
    camelize, get_bind_name, get_bind_value, has_attr_modifier, has_camel_modifier,
    has_prop_modifier, is_dynamic_binding, process_v_bind,
};
pub use v_for::{
    get_for_expression, has_v_for, parse_for_expression, parse_for_expression_with_options,
    process_v_for, remove_for_directive,
};
pub use v_if::{
    get_if_condition, has_v_else, has_v_else_if, has_v_if, process_v_if, remove_if_directive,
};
pub use v_memo::{
    MemoInfo, generate_memo_check, generate_v_memo_wrapper, get_memo_deps, get_memo_exp,
    has_v_memo, process_v_memo, remove_v_memo,
};
pub use v_model::{
    VModelModifiers, get_model_event_prop, get_vmodel_helper, parse_model_modifiers,
    supports_v_model, transform_v_model,
};
pub use v_on::{
    EventModifiers, create_on_name, get_event_name, get_handler_expression, is_dynamic_event,
    needs_guard, parse_event_modifiers, process_v_on,
};
pub use v_once::{generate_v_once_wrapper, has_v_once, remove_v_once, transform_v_once};
pub use v_slot::{
    SlotInfo, SlotOutletInfo, collect_slots, get_slot_name, get_slot_props_string,
    has_dynamic_slots, has_v_slot, is_dynamic_slot, transform_slot_outlet,
};
