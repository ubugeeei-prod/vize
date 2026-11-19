//! Reusable Vue syntax features composed by the per-version tables.

use super::{DirectiveArgStyle, LegacyDialectCapabilities};

pub mod attribute;
pub mod component;
pub mod computed;
pub mod directive;
pub mod event;
pub mod filters;
pub mod inheritance;
pub mod repeat;
pub mod slot_scope;
pub mod triple_mustache;

/// The Vue 3 baseline before any historical feature is composed.
pub const BASE: LegacyDialectCapabilities = LegacyDialectCapabilities {
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
    duplicate_static_attrs_keep_last: false,
};
