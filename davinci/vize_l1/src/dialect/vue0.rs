//! Vue 0 template capability table.

use super::vue::{DirectiveArgStyle, LegacyDialectCapabilities};

/// Existing V0_10 template syntax, resolved once per file.
pub const V0_10: LegacyDialectCapabilities = LegacyDialectCapabilities {
    supports_filters: true,
    space_separated_filter_args: true,
    v_repeat_syntax: true,
    directive_arg_style: DirectiveArgStyle::Clause,
    v_with_directive: true,
    v_component_directive: true,
    computed_dollar_get_set: true,
    attr_value_interpolation: true,
    scoped_slot_attrs: false,
    raw_html_interpolation: true,
    v2_event_sugar: false,
};
/// Existing V0_11 template syntax, resolved once per file.
pub const V0_11: LegacyDialectCapabilities = LegacyDialectCapabilities {
    supports_filters: true,
    space_separated_filter_args: true,
    v_repeat_syntax: true,
    directive_arg_style: DirectiveArgStyle::Clause,
    v_with_directive: true,
    v_component_directive: true,
    computed_dollar_get_set: false,
    attr_value_interpolation: true,
    scoped_slot_attrs: false,
    raw_html_interpolation: true,
    v2_event_sugar: false,
};
