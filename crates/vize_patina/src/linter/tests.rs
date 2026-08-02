use super::Linter;
use crate::LintPreset;
use vize_carton::{Allocator, ToCompactString};

mod basic;
mod css;
mod directives;
mod jsx;
mod jsx_fallback;
mod no_mutating_props;
mod no_top_level_ref;
mod nuxt;
mod nuxt_config_order;
mod nuxt_prefer_nuxt_link;
mod recovered_parse_analysis;
mod script;
mod severity_overrides;
mod sfc;
mod v_for_unused_vars;
