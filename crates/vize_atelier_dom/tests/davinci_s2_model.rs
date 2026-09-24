//! P2-11 `v-model` witness: native `withDirectives` + `vModelText`-family
//! helpers, component `modelValue` / `onUpdate:` product props, compared
//! **byte-for-byte** including helper usage and hoists.

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

#[test]
fn s2_v_model_matches_the_shipped_dom_lane_byte_for_byte() {
    support::assert_s2_matches_shipped(support::battery::model::MODEL_BATTERY);
}
