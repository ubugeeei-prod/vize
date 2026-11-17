use crate::support::fix_history_runner::check_pack;

#[test]
fn disabled_prop_checks_skip_component_prop_assignability() {
    check_pack(
        "no-check-props-component-props-569",
        "fix_history_cases::no_check_props_component_props_569::disabled_prop_checks_skip_component_prop_assignability",
        "67ec25621c3d4ccbcd36215a7f701bc3e3fb247e",
        Some(569),
        &[
            "mismatched-component-prop-unchecked",
            "template-expression-still-checked",
        ],
    );
}
