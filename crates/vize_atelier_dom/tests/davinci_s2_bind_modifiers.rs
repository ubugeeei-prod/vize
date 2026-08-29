//! P2-11 `v-bind` modifier witness: static-name `.camel`, `.prop`,
//! `.attr`, and the dot shorthand compare byte-for-byte against the
//! shipped DOM lane.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

mod support;

const BATTERY: &[(&str, &str)] = &[
    ("native_camel", r#"<div :foo-bar.camel="value"></div>"#),
    ("native_prop", r#"<div :value.prop="value"></div>"#),
    ("native_attr", r#"<div :value.attr="value"></div>"#),
    ("dot_shorthand", r#"<div .value="value"></div>"#),
    ("component_camel", r#"<Foo :foo-bar.camel="value" />"#),
    ("component_prop", r#"<Foo :value.prop="value" />"#),
    (
        "merge_props_camel",
        r#"<div v-bind="bag" :foo-bar.camel="value"></div>"#,
    ),
    (
        "merge_props_prop",
        r#"<div v-bind="bag" :value.prop="value"></div>"#,
    ),
    ("object_prop", r#"<div v-bind.prop="bag"></div>"#),
    ("object_attr", r#"<div v-bind.attr="bag"></div>"#),
    ("object_camel", r#"<div v-bind.camel="bag"></div>"#),
    (
        "merge_object_prop",
        r#"<div id="x" v-bind.prop="bag"></div>"#,
    ),
    ("v_if_attr", r#"<div v-if="ok" :value.attr="value"></div>"#),
    (
        "v_for_camel",
        r#"<div v-for="item in items" :key="item.id" :foo-bar.camel="item.value"></div>"#,
    ),
    ("slot_outlet_prop", r#"<slot :foo-bar.prop="value" />"#),
];

#[test]
fn s2_bind_modifiers_match_the_shipped_dom_lane_byte_for_byte() {
    support::assert_s2_matches_shipped(BATTERY);
}
