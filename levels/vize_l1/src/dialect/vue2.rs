//! Vue 2 template capability table.

pub mod surface;
pub mod text;

use super::vue::{DirectiveArgStyle, LegacyDialectCapabilities};

/// Existing V2 template syntax, resolved once per file.
pub const CAPABILITIES: LegacyDialectCapabilities = LegacyDialectCapabilities {
    supports_filters: true,
    space_separated_filter_args: false,
    v_repeat_syntax: false,
    directive_arg_style: DirectiveArgStyle::Colon,
    v_with_directive: false,
    v_component_directive: false,
    computed_dollar_get_set: false,
    attr_value_interpolation: false,
    scoped_slot_attrs: true,
    raw_html_interpolation: false,
    v2_event_sugar: true,
};
