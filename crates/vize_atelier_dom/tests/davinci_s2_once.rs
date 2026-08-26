//! P2-11 `v-once` witness: native element cache wrappers from S2,
//! compared **byte-for-byte** against the shipped DOM lane.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

mod support;

const BATTERY: &[(&str, &str)] = &[
    ("simple", r#"<div v-once>x</div>"#),
    ("interpolation", r#"<div v-once>{{ msg }}</div>"#),
    ("dynamic_class", r#"<div v-once :class="cls">content</div>"#),
    (
        "dynamic_style",
        r#"<div v-once :style="style">content</div>"#,
    ),
    ("nested_static", r#"<div v-once><span>x</span></div>"#),
    (
        "nested_dynamic",
        r#"<div v-once><span :class="cls">{{ msg }}</span></div>"#,
    ),
    ("v_if_branch", r#"<div v-if="ok" v-once>x</div>"#),
    (
        "inside_v_for",
        r#"<div v-for="item in items" :key="item.id"><span v-once>{{ item.static }}</span></div>"#,
    ),
];

#[test]
fn s2_v_once_matches_the_shipped_dom_lane_byte_for_byte() {
    support::assert_s2_matches_shipped(BATTERY);
}
