use crate::support::fix_history_runner::check_pack;

#[test]
fn disabled_prop_checks_skip_keyed_template_prop_constraints() {
    check_pack(
        "no-check-props-keyed-template-props",
        "fix_history_cases::no_check_props_keyed_template_props::disabled_prop_checks_skip_keyed_template_prop_constraints",
        "3e7efb82ad29fb18eee89e31bcc5bf0053f2b409",
        None,
        &[
            "undeclared-keyed-prop-unchecked",
            "declared-prop-wrong-member",
        ],
    );
}
