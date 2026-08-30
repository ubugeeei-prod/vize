//! Root static-props hoist position gates, compared byte-for-byte.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

mod support;

const BATTERY: &[(&str, &str)] = &[
    (
        "root_static_props_with_component_child",
        r#"<div class="wrapper"><Foo /></div>"#,
    ),
    (
        "root_static_props_with_static_nested_dynamic_text",
        r#"<div class="wrapper"><span>{{ msg }}</span></div>"#,
    ),
    (
        "component_static_props_with_component_slot",
        r#"<Foo class="panel"><Bar /></Foo>"#,
    ),
];

#[test]
fn s2_root_hoist_position_gates_match_the_shipped_dom_lane() {
    support::assert_s2_matches_shipped(BATTERY);
}
