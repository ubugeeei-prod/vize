use crate::support::fix_history_runner::check_pack;

#[test]
fn with_defaults_keeps_optional_boolean_props_normalized() {
    check_pack(
        "with-defaults-boolean-normalization-5724",
        "fix_history_cases::with_defaults_boolean_normalization_5724::with_defaults_keeps_optional_boolean_props_normalized",
        "9bf9ee47adb4da3a29eb049c18588d4f917a440c",
        Some(5724),
        &[
            "with-defaults-keeps-untouched-boolean-normalized",
            "with-defaults-boolean-read-into-string-ref",
            "with-defaults-boolean-props-in-template",
        ],
    );
}
