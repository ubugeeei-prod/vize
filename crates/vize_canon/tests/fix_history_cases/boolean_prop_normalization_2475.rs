use crate::support::fix_history_runner::check_pack;

#[test]
fn optional_boolean_props_normalize_to_boolean() {
    check_pack(
        "boolean-prop-normalization-2475",
        "fix_history_cases::boolean_prop_normalization_2475::optional_boolean_props_normalize_to_boolean",
        "87ed46d9a282a5f3da07900c6c67e5a7b652a316",
        Some(2475),
        &[
            "optional-boolean-prop-normalized-in-setup",
            "optional-string-prop-not-normalized",
            "with-defaults-boolean-prop-normalized",
            "with-defaults-string-prop-not-boolean",
        ],
    );
}
