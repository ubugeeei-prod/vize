//! P2-11 static-cache witness: when a root hoist enables the legacy
//! `_cache` gate, sibling static element children are cached as one array.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

mod support;

const BATTERY: &[(&str, &str)] = &[(
    "hoisted_root_caches_static_children_array",
    r#"<div class="root"><span>a</span><span>b</span></div>"#,
)];

#[test]
fn s2_static_child_cache_matches_the_shipped_dom_lane_byte_for_byte() {
    support::assert_s2_matches_shipped(BATTERY);
}
