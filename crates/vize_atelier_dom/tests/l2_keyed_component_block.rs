//! Nested `:key` components stay block VNodes, matching the shipped DOM lane.

use crate::support;

const BATTERY: &[(&str, &str)] = &[(
    "slot_nested_keyed_component",
    r#"<Story><template #controls><div p-4><RippleGrid :key="renderKey" /></div></template></Story>"#,
)];

#[test]
fn l2_keyed_nested_components_match_the_shipped_dom_lane_byte_for_byte() {
    support::assert_l2_matches_shipped(BATTERY);
}
