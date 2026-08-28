//! P2-11 `v-memo` witness: cache wrappers and `v-for` cached-item
//! guards, compared **byte-for-byte** including helper usage.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

mod support;

const BATTERY: &[(&str, &str)] = &[
    ("native_text", r#"<div v-memo="[id]">x</div>"#),
    (
        "native_interpolation",
        r#"<div v-memo="[id]">{{ msg }}</div>"#,
    ),
    (
        "native_dynamic_prop",
        r#"<div v-memo="[id]" :id="id">{{ msg }}</div>"#,
    ),
    (
        "nested_native",
        r#"<div><span v-memo="[id]">{{ msg }}</span></div>"#,
    ),
    ("component_root", r#"<Foo v-memo="[prop]" :prop="prop" />"#),
    (
        "nested_component",
        r#"<div><Foo v-memo="[prop]" :prop="prop" /></div>"#,
    ),
    ("native_v_if", r#"<div v-if="ok" v-memo="[id]">x</div>"#),
    (
        "component_v_if",
        r#"<Foo v-if="ok" v-memo="[prop]" :prop="prop" />"#,
    ),
    (
        "native_v_for",
        r#"<div v-for="item in items" :key="item.id" v-memo="[item.selected]">{{ item.name }}</div>"#,
    ),
    (
        "component_v_for",
        r#"<Foo v-for="item in items" :key="item.id" v-memo="[item.selected]" :prop="item.prop" />"#,
    ),
    (
        "unkeyed_v_for",
        r#"<div v-for="item in items" v-memo="[item.selected]">{{ item.name }}</div>"#,
    ),
    (
        "numeric_v_for",
        r#"<div v-for="n in 3" v-memo="[n]">{{ n }}</div>"#,
    ),
];

#[test]
fn s2_v_memo_matches_the_shipped_dom_lane_byte_for_byte() {
    support::assert_s2_matches_shipped(BATTERY);
}
