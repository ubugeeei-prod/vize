//! P2-11 `.native` event-sugar witness: the Vue 2 modifier is accepted
//! and stripped before event-key calculation / handler wrapping, compared
//! **byte-for-byte** against the shipped DOM lane.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

mod support;

const BATTERY: &[(&str, &str)] = &[
    ("native_click", r#"<div @click.native="h"></div>"#),
    ("native_stop", r#"<div @click.native.stop="h"></div>"#),
    ("stop_native", r#"<div @click.stop.native="h"></div>"#),
    ("native_once", r#"<div @click.native.once="h"></div>"#),
    ("native_key", r#"<div @keyup.native.enter="h"></div>"#),
    ("native_inline", r#"<div @click.native="count++"></div>"#),
    ("component_native", r#"<Foo @click.native="h" />"#),
    ("component_native_stop", r#"<Foo @click.native.stop="h" />"#),
    (
        "v_if_native",
        r#"<button v-if="ok" @click.native="h"></button>"#,
    ),
    (
        "v_for_native",
        r#"<button v-for="item in items" :key="item.id" @click.native="h">{{ item.label }}</button>"#,
    ),
    (
        "native_duplicate",
        r#"<div @click.native="a" @click="b"></div>"#,
    ),
    (
        "native_duplicate_stop",
        r#"<div @click.native="a" @click.stop="b"></div>"#,
    ),
];

#[test]
fn s2_native_event_sugar_matches_the_shipped_dom_lane_byte_for_byte() {
    support::assert_s2_matches_shipped(BATTERY);
}
