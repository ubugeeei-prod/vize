//! Vue 1 template capability table.

use super::vue::{DirectiveArgStyle, LegacyDialectCapabilities};

/// Existing V1 template syntax, resolved once per file.
pub const CAPABILITIES: LegacyDialectCapabilities = LegacyDialectCapabilities {
    supports_filters: true,
    space_separated_filter_args: true,
    v_repeat_syntax: false,
    directive_arg_style: DirectiveArgStyle::Colon,
    v_with_directive: false,
    v_component_directive: false,
    computed_dollar_get_set: false,
    attr_value_interpolation: true,
    scoped_slot_attrs: false,
    raw_html_interpolation: true,
    v2_event_sugar: false,
};
