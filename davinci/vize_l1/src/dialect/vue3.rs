//! Default Vue 3 template capability table.

pub mod directive;
pub use directive::VueDirectives;

use super::vue::{DirectiveArgStyle, LegacyDialectCapabilities};

/// No legacy syntax; colon-style directive arguments.
pub const CAPABILITIES: LegacyDialectCapabilities = LegacyDialectCapabilities {
    supports_filters: false,
    space_separated_filter_args: false,
    v_repeat_syntax: false,
    directive_arg_style: DirectiveArgStyle::Colon,
    v_with_directive: false,
    v_component_directive: false,
    computed_dollar_get_set: false,
    attr_value_interpolation: false,
    scoped_slot_attrs: false,
    raw_html_interpolation: false,
    v2_event_sugar: false,
};
