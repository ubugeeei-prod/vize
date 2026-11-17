use crate::support::fix_history_runner::check_pack;

#[test]
fn with_defaults_setup_props_are_template_bindings() {
    check_pack(
        "setup-props-template-scope-2081",
        "fix_history_cases::setup_props_template_scope_2081::with_defaults_setup_props_are_template_bindings",
        "1604bd9c3a17f78bdec64433d9aebde4ca4641f8",
        Some(2081),
        &[
            "with-defaults-interface-props-in-template",
            "union-member-is-not-a-template-binding",
            "with-defaults-inline-props-in-template",
            "inline-prop-member-wrong-type",
        ],
    );
}
