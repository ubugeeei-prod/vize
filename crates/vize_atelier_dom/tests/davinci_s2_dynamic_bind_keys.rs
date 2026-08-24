//! P2-11 dynamic `v-bind` key witness: computed prop keys and their
//! `.camel` / `.prop` / `.attr` modifier forms compare byte-for-byte
//! against the shipped DOM lane.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

mod support;

const BATTERY: &[(&str, &str)] = &[
    ("native_dynamic", r#"<div :[key]="value"></div>"#),
    (
        "native_dynamic_camel",
        r#"<div :[key].camel="value"></div>"#,
    ),
    ("native_dynamic_prop", r#"<div :[key].prop="value"></div>"#),
    ("native_dynamic_attr", r#"<div :[key].attr="value"></div>"#),
    ("component_dynamic", r#"<Foo :[key]="value" />"#),
    (
        "merge_props_dynamic",
        r#"<div v-bind="bag" :[key]="value"></div>"#,
    ),
    ("v_if_dynamic", r#"<div v-if="ok" :[key]="value"></div>"#),
    (
        "v_for_dynamic",
        r#"<div v-for="item in items" :[key]="item.value"></div>"#,
    ),
    ("slot_outlet_dynamic", r#"<slot :[key]="value" />"#),
    (
        "slot_outlet_dynamic_camel",
        r#"<slot :[key].camel="value" />"#,
    ),
];

#[test]
fn s2_dynamic_bind_keys_match_the_shipped_dom_lane_byte_for_byte() {
    support::assert_s2_matches_shipped(BATTERY);
}
