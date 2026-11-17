use crate::support::fix_history_runner::check_pack;

#[test]
fn whole_props_check_target_displays_the_public_props_shape() {
    check_pack(
        "whole-props-check-display-3906",
        "fix_history_cases::whole_props_check_display_3906::whole_props_check_target_displays_the_public_props_shape",
        "1234f20675cbb1c13a2815afa8ac3a957defb244",
        Some(3906),
        &[
            "generic-emit-overloads-missing-required-count",
            "generic-emit-overloads-complete-props",
            "sfc-emit-listener-missing-required-count",
            "sfc-emit-listener-complete-props",
            "sfc-emit-listener-prop-wrong-callback",
        ],
    );
}
