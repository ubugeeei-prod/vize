//! `v-pre` DOM parity witnesses.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

mod support;

const BATTERY: &[(&str, &str)] = &[
    (
        "interpolation_renders_as_static_text",
        r#"<div v-pre class="code">{{ variable }}</div>"#,
    ),
    (
        "multiline_interpolation_condenses_as_static_text",
        r#"<code v-pre class="font-code">
  {{ variable }}
</code>"#,
    ),
];

#[test]
fn s2_v_pre_matches_the_shipped_dom_lane_byte_for_byte() {
    support::assert_s2_matches_shipped(BATTERY);
}
