//! Default Vue 3 template capability table.

pub mod directive;
pub(crate) mod for_head;
pub(crate) mod operand;
pub mod surface;
pub use directive::VueDirectives;

/// Component classification for the default Vue lint policy.
///
/// Uppercase tags include the core built-ins. The lowercase dynamic component
/// is Vue syntax only while its header is active; exact or inherited `v-pre`
/// leaves it literal. Configured native/custom element resolution is separate.
pub fn is_lint_component(tag: &str, dynamic_component: bool) -> bool {
    (dynamic_component && tag == "component") || tag.chars().next().is_some_and(char::is_uppercase)
}

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
