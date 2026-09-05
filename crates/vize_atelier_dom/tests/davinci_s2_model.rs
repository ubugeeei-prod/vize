//! P2-11 `v-model` witness: native `withDirectives` + `vModelText`-family
//! helpers, component `modelValue` / `onUpdate:` product props, compared
//! **byte-for-byte** including helper usage and hoists.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

mod support;

#[test]
fn s2_v_model_matches_the_shipped_dom_lane_byte_for_byte() {
    support::assert_s2_matches_shipped(support::battery::model::MODEL_BATTERY);
}
