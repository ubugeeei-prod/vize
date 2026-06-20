//! Extraction functions for props, emits, and reactivity detection.

mod common;
mod emits;
mod exports;
mod macros;
mod plain_values;
mod props;
mod props_type;
mod provide;
mod race;
mod reactivity;
mod runtime_objects;
mod slots;

pub use common::{extract_call_expression, get_binding_type_from_kind};
pub(in crate::script_parser) use emits::extract_runtime_emit_payload_type;
pub use exports::{process_invalid_export, process_type_export};
pub use macros::process_call_expression;
pub(in crate::script_parser) use plain_values::reactive_destructure_source;
pub use plain_values::{
    check_getter_call_extraction, check_reactive_plain_alias_extraction,
    check_reactive_plain_assignment_alias, check_reactive_plain_assignment_mutation,
    check_reactive_plain_call_mutation, check_reactive_plain_update_mutation,
    check_reactive_property_extraction, check_reactive_spread_expression,
    check_ref_value_extraction, detect_call_argument_reactivity_loss,
    record_getter_context_from_call,
};
pub use provide::{detect_provide_inject_call, extract_argument_source, extract_provide_key};
pub use race::detect_race_condition_call;
pub use reactivity::{detect_reactivity_call, detect_setup_context_violation};
pub(in crate::script_parser) use runtime_objects::record_static_runtime_object_literal;
