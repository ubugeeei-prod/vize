use crate::support::fix_history_runner::check_pack;

#[test]
fn with_defaults_retains_loose_required_props() {
    check_pack(
        "with-defaults-loose-required-2924",
        "fix_history_cases::with_defaults_loose_required_2924::with_defaults_retains_loose_required_props",
        "a619bad6a42dc8c6e3cbd4f830a2e691a64c5cef",
        Some(2924),
        &[
            "defaulted-template-props",
            "loose-required-undefaulted-key",
            "undefaulted-prop-possibly-undefined",
        ],
    );
}
