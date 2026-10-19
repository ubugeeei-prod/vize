//! P2-11 `v-model` witness: native `withDirectives` + `vModelText`-family
//! helpers, component `modelValue` / `onUpdate:` product props, compared
//! **byte-for-byte** including helper usage and hoists.

use crate::support;

#[test]
fn l2_v_model_matches_the_shipped_dom_lane_byte_for_byte() {
    support::assert_l2_matches_shipped(support::battery::model::MODEL_BATTERY);
}
