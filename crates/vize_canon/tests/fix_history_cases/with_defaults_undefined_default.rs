use crate::support::fix_history_runner::check_pack;

#[test]
fn with_defaults_undefined_default_stays_optional() {
    check_pack(
        "with-defaults-undefined-default",
        "fix_history_cases::with_defaults_undefined_default::with_defaults_undefined_default_stays_optional",
        "38b5adbedf092bca14579a827e1af4f4b28ad03c",
        None,
        &[
            "undefined-default-keeps-optional-value",
            "undefined-default-unguarded-clone-error",
            "undefined-default-template-binding-not-narrowed",
        ],
    );
}
