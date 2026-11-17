use crate::support::fix_history_runner::check_pack;

#[test]
fn with_defaults_undefined_boolean_keeps_controlled_vmodel_overloads() {
    check_pack(
        "with-defaults-undefined-boolean-vmodel",
        "fix_history_cases::with_defaults_undefined_boolean_vmodel::with_defaults_undefined_boolean_keeps_controlled_vmodel_overloads",
        "f647dbb6698c3125e95c12ed1c5d947d08f600b0",
        None,
        &[
            "undefined-boolean-default-vmodel-overloads",
            "wrong-open-default-value",
            "undefined-boolean-default-stays-optional",
        ],
    );
}
