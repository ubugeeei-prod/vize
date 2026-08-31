//! Static object `:style` output, compared byte-for-byte against the
//! shipped DOM lane.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

mod support;

const BATTERY: &[(&str, &str)] = &[
    (
        "nested_native_static_object_style",
        r#"<section><div :style="{ color: 'red' }"></div></section>"#,
    ),
    (
        "branch_native_static_object_style",
        r#"<section><div v-if="ok" :style="{ color: 'red' }"></div></section>"#,
    ),
    (
        "native_dynamic_identifier_style",
        r#"<section><div :style="s"></div></section>"#,
    ),
    (
        "native_static_and_dynamic_style_merge",
        r#"<section><div style="color: red" :style="s"></div></section>"#,
    ),
    (
        "native_computed_key_style",
        r#"<section><div :style="{ [prop]: 'red' }"></div></section>"#,
    ),
    (
        "component_static_object_style",
        r#"<section><Foo :style="{ color: 'red' }" /></section>"#,
    ),
    (
        "component_dynamic_identifier_style",
        r#"<section><Foo :style="s" /></section>"#,
    ),
    (
        "component_computed_key_style",
        r#"<section><Foo :style="{ [prop]: 'red' }" /></section>"#,
    ),
    (
        "component_static_and_dynamic_style_merge",
        r#"<section><Foo style="color: red" :style="s" /></section>"#,
    ),
    (
        "component_static_and_static_object_style_merge",
        r#"<section><Foo style="color: red" :style="{ top: '1px' }" /></section>"#,
    ),
];

#[test]
fn s2_static_object_style_matches_the_shipped_dom_lane_byte_for_byte() {
    support::assert_s2_matches_shipped(BATTERY);
}
