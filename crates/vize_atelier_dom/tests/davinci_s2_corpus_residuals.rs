//! DOM corpus residuals promoted to focused byte-for-byte pins.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

mod support;

const BATTERY: &[(&str, &str)] = &[
    (
        "foreign_svg_static_bind_root_splits_props_and_child_hoist",
        r#"<Handle><svg viewBox="0 0 16 16" fill="none" xmlns="http://www.w3.org/2000/svg" :width="10" :height="10"><circle cx="8" cy="8" r="7" fill="hotpink" /></svg></Handle>"#,
    ),
    (
        "component_slot_static_global_constant_prop_uses_props_flag",
        r#"<md-tabs><md-tab :id="NaN" md-label="Tab id=NaN">NaN</md-tab></md-tabs>"#,
    ),
    (
        "transition_slot_props_fallback_keeps_static_props_inline",
        r#"<transition name="sw-update-popup"><slot :reload="reload"><br /></slot></transition>"#,
    ),
    (
        "transition_named_slot_fallback_hoists_static_props",
        r#"<transition name="multiselect__loading"><slot name="loading"><div v-show="loading" class="multiselect__spinner"></div></slot></transition>"#,
    ),
    (
        "v_else_with_value_uses_legacy_else_if_condition",
        r#"<div v-if="disabled">disabled</div><span v-else="enabled">enabled</span>"#,
    ),
    (
        "foreign_defs_with_static_bound_gradient_caches_as_legacy",
        r##"<svg class="va-icon-vuestic"><defs><linearGradient :id="'ORIGINAL'" x1="0%" y1="50%" y2="50%"><stop offset="0%" stop-color="#4AE387" /><stop offset="100%" stop-color="#C8EA13" /></linearGradient></defs><path :fill="textColor" /></svg>"##,
    ),
];

#[test]
fn s2_dom_corpus_residuals_match_the_shipped_lane() {
    support::assert_s2_matches_shipped(BATTERY);
}
