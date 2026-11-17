use crate::support::fix_history_runner::check_pack;

#[test]
fn generic_callback_prop_listener_infers_payload() {
    check_pack(
        "generic-callback-listener-props-2921",
        "fix_history_cases::generic_callback_listener_props_2921::generic_callback_prop_listener_infers_payload",
        "e3a12cf323cb3f7213034e5137564ca4daf794b1",
        Some(2921),
        &[
            "callback-prop-listener-inferred",
            "callback-prop-listener-payload-mismatch",
        ],
    );
}
