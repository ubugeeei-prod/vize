//! P2-11 `v-if` / `v-for` co-carrier branch emission.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

mod support;

const BATTERY: &[(&str, &str)] = &[
    (
        "native_unkeyed",
        r#"<div v-if="ok" v-for="i in n">{{ i }}</div>"#,
    ),
    (
        "native_keyed",
        r#"<div v-if="ok" v-for="i in n" :key="i">{{ i }}</div>"#,
    ),
    (
        "component_keyed",
        r#"<Foo v-if="ok" v-for="item in list" :key="item" />"#,
    ),
];

#[test]
fn s2_if_for_co_carriers_match_the_shipped_dom_lane_byte_for_byte() {
    support::assert_s2_matches_shipped(BATTERY);
}
