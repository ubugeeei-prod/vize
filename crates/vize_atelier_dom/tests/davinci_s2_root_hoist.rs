//! Root static-props hoist position gates, compared byte-for-byte.

#[expect(
    dead_code,
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::string_slice,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "shared test support; each binary uses a subset"
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
    (
        "root_static_props_with_slotted_component_child",
        r#"<div id="root"><Foo id="x">hello</Foo></div>"#,
    ),
];

#[test]
fn s2_root_hoist_position_gates_match_the_shipped_dom_lane() {
    support::assert_s2_matches_shipped(BATTERY);
}
