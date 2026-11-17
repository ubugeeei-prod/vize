use crate::support::fix_history_runner::check_pack;

#[test]
fn generic_child_inline_callback_prop_is_contextually_typed() {
    check_pack(
        "generic-inline-callback-prop-3511",
        "fix_history_cases::generic_inline_callback_prop_3511::generic_child_inline_callback_prop_is_contextually_typed",
        "51253bbae73236b3710765f11651f9c32e72ffcb",
        Some(3511),
        &[
            "inline-callback-prop-contextually-typed",
            "inline-callback-prop-mismatches",
            "generic-list-selected-mismatch",
        ],
    );
}
