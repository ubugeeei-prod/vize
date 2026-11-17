use crate::support::fix_history_runner::check_pack;

#[test]
fn optional_boolean_template_prop_is_normalized() {
    check_pack(
        "optional-boolean-template-props-2731",
        "fix_history_cases::optional_boolean_template_props_2731::optional_boolean_template_prop_is_normalized",
        "a76940eb95b9fdbba90331b5b3b1bab2331ec902",
        Some(2731),
        &[
            "optional-boolean-to-required-boolean",
            "normalized-boolean-to-string-error",
        ],
    );
}
