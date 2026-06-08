//! Extraction functions for props, emits, and reactivity detection.

mod common;
mod emits;
mod exports;
mod macros;
mod plain_values;
mod props;
mod provide;
mod race;
mod reactivity;
mod slots;

pub use common::{extract_call_expression, get_binding_type_from_kind};
pub use exports::{process_invalid_export, process_type_export};
pub use macros::process_call_expression;
pub use plain_values::{
    check_getter_call_extraction, check_reactive_plain_alias_extraction,
    check_reactive_plain_assignment_alias, check_reactive_property_extraction,
    check_ref_value_extraction, detect_call_argument_reactivity_loss,
    record_getter_context_from_call,
};
pub use provide::{detect_provide_inject_call, extract_argument_source, extract_provide_key};
pub use race::detect_race_condition_call;
pub use reactivity::{detect_reactivity_call, detect_setup_context_violation};
