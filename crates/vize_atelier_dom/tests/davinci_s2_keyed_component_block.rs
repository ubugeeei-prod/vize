//! Nested `:key` components stay block VNodes, matching the shipped DOM lane.

#[expect(
    dead_code,
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::string_slice,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "shared test support; each binary uses a subset"
)]
mod support;

const BATTERY: &[(&str, &str)] = &[(
    "slot_nested_keyed_component",
    r#"<Story><template #controls><div p-4><RippleGrid :key="renderKey" /></div></template></Story>"#,
)];

#[test]
fn s2_keyed_nested_components_match_the_shipped_dom_lane_byte_for_byte() {
    support::assert_s2_matches_shipped(BATTERY);
}
