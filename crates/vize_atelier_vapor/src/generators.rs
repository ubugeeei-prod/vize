//! Vapor code generators.
//!
//! Individual generator modules for Vapor code generation.

pub mod block;
pub mod component;
pub mod directive;
pub mod event;
pub mod for_node;
pub mod generate_slot;
pub mod generate_text;
pub mod if_node;
pub mod prop;

pub use block::{
    escape_template, generate_block, generate_effect_wrapper, generate_template_declaration,
    generate_template_instantiation, GenerateContext,
};
pub use component::{
    generate_async_component, generate_create_component, generate_dynamic_component,
    generate_keep_alive, generate_resolve_component, generate_suspense,
};
pub use directive::{
    generate_directive, generate_directive_array, generate_resolve_directive,
    generate_v_cloak_removal, generate_v_show, generate_with_directives, is_v_pre_element,
};
pub use event::{
    capitalize_event_name, generate_delegate_event, generate_event_options,
    generate_inline_handler, generate_set_event,
};
pub use for_node::{can_optimize_for, generate_for, generate_for_memo};
pub use generate_slot::{
    generate_dynamic_slot_name, generate_normalize_slots, generate_scoped_slots,
    generate_slot_function, generate_slot_outlet, is_dynamic_slot_name,
};
pub use generate_text::{
    build_text_expression, can_inline_text, generate_create_text_node, generate_set_text,
    generate_text_content, generate_to_display_string,
};
pub use if_node::{can_use_ternary, generate_if, generate_if_expression};
pub use prop::{
    generate_attribute, generate_class_binding, generate_component_prop,
    generate_set_dynamic_props, generate_set_prop, generate_style_binding, normalize_prop_key,
};
