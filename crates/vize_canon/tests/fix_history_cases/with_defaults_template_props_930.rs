use crate::support::fix_history_runner::check_pack;

#[test]
fn with_defaults_template_props_are_default_resolved() {
    check_pack(
        "with-defaults-template-props-930",
        "fix_history_cases::with_defaults_template_props_930::with_defaults_template_props_are_default_resolved",
        "bea7cd0664ef06b9779ac00708501230ca2897d9",
        Some(930),
        &[
            "defaulted-props-are-required-in-template",
            "non-defaulted-optional-prop-stays-optional",
        ],
    );
}
