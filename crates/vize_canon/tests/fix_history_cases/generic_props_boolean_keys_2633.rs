use crate::support::fix_history_runner::check_pack;

#[test]
fn generic_props_do_not_gain_boolean_intersections() {
    check_pack(
        "generic-props-boolean-keys-2633",
        "fix_history_cases::generic_props_boolean_keys_2633::generic_props_do_not_gain_boolean_intersections",
        "3c2edcdebd2f836c3e96a8447f57e000de453315",
        Some(2633),
        &[
            "with-defaults-generic-value-without-boolean-intersection",
            "with-defaults-string-prop-argument-type-error",
            "local-boolean-key-preserved",
            "local-boolean-key-string-type-error",
        ],
    );
}
