use crate::support::fix_history_runner::check_pack;

#[test]
fn dynamic_v_slot_props_use_declared_slot_payloads() {
    check_pack(
        "dynamic-v-slot-props-1260",
        "fix_history_cases::dynamic_v_slot_props_1260::dynamic_v_slot_props_use_declared_slot_payloads",
        "9f8a92fe20897915dfc2e1f134b7a66cb09d26c4",
        Some(1260),
        &[
            "dynamic-slot-prop-used-as-string",
            "dynamic-slot-prop-wrong-member",
            "dynamic-slot-name-selects-payload",
            "dynamic-slot-name-selects-other-payload",
        ],
    );
}
