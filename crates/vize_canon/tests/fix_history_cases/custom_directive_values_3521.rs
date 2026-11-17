use crate::support::fix_history_runner::check_pack;

#[test]
fn custom_directive_values_are_checked_against_directive_value_type() {
    check_pack(
        "custom-directive-values-3521",
        "fix_history_cases::custom_directive_values_3521::custom_directive_values_are_checked_against_directive_value_type",
        "375b0fb5b0d04af9014d81ef865d17dba02f3b0a",
        Some(3521),
        &[
            "typed-directive-value-matches",
            "typed-directive-value-mismatch",
            "unresolvable-and-untyped-directives",
            "directive-value-member-valid",
            "directive-value-missing-member",
        ],
    );
}
