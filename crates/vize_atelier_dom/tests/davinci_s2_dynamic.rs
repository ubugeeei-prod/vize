//! P2-11 `<component :is>` witness: `resolveDynamicComponent`, skipped
//! `is` props / patch flags, nested `createBlock`, compared
//! **byte-for-byte** including helper usage.

#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods,
    reason = "insta and fixtures use format!; fixtures use std strings"
)]

#[expect(
    dead_code,
    clippy::string_slice,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "shared test support; each binary uses a subset"
)]
mod support;

const BATTERY: &[(&str, &str)] = &[
    ("dynamic_bind", r#"<component :is="x" />"#),
    ("dynamic_empty", r#"<component :is="x"></component>"#),
    ("static_is", r#"<component is="Foo" />"#),
    ("pascal_bind", r#"<Component :is="x" />"#),
    ("pascal_static", r#"<Component is="Foo" />"#),
    ("bare_component", "<component />"),
    ("with_id", r#"<component :is="x" id="a" />"#),
    ("with_bind", r#"<component :is="x" :foo="bar" />"#),
    ("text_slot", r#"<component :is="x">hello</component>"#),
    (
        "span_slot",
        r#"<component :is="x"><span></span></component>"#,
    ),
    ("nested", r#"<div><component :is="x" /></div>"#),
    ("vif", r#"<component v-if="ok" :is="x" />"#),
    ("vfor", r#"<component v-for="i in n" :is="x" />"#),
    ("object_bind", r#"<component :is="x" v-bind="obj" />"#),
    ("in_slot", r#"<Foo><component :is="x" /></Foo>"#),
    (
        "static_is_slot",
        r#"<component is="Foo" id="a">hello</component>"#,
    ),
    ("quoted", r#"<component :is="'Foo'" />"#),
];

#[test]
fn s2_dynamic_is_matches_the_shipped_dom_lane_byte_for_byte() {
    support::assert_s2_matches_shipped(BATTERY);
}
