//! `v-pre` DOM parity witnesses.

use crate::support;

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
fn l2_v_pre_matches_the_shipped_dom_lane_byte_for_byte() {
    support::assert_l2_matches_shipped(BATTERY);
}
