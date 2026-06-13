//! Vapor transform steps.
//!
//! Individual transform steps for Vapor IR generation.

#[path = "transforms/element.rs"]
pub mod element;
#[path = "transforms/transform_slot.rs"]
pub mod slot;
#[path = "transforms/transform_text.rs"]
pub mod text;
#[path = "transforms/v_bind.rs"]
pub mod v_bind;
#[path = "transforms/v_for.rs"]
pub mod v_for;
#[path = "transforms/v_if.rs"]
pub mod v_if;
#[path = "transforms/v_model.rs"]
pub mod v_model;
#[path = "transforms/v_on.rs"]
pub mod v_on;
#[path = "transforms/v_show.rs"]
pub mod v_show;

pub use element::{
    generate_element_template, get_tag_name, has_dynamic_bindings, has_event_listeners,
    is_component, is_slot_outlet, is_static_element, is_template_wrapper,
};
pub use slot::{collect_component_slots, transform_slot_outlet};
pub use text::{
    generate_text_expression, should_merge_text_nodes, transform_interpolation, transform_text,
};
pub use v_bind::{is_dynamic_binding, transform_v_bind, transform_v_bind_dynamic};
pub use v_for::{parse_for_alias, transform_for_node, transform_v_for};
pub use v_if::{transform_if_branches, transform_v_if};
pub use v_model::{
    generate_model_handler, get_model_arg, get_model_event, get_model_modifiers, get_model_value,
    has_lazy_modifier, has_number_modifier, has_trim_modifier, transform_v_model,
};
pub use v_on::{generate_event_handler, transform_v_on};
pub use v_show::{generate_v_show_effect, get_show_condition, needs_transition, transform_v_show};
