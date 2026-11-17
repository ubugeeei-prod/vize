use crate::support::fix_history_runner::check_pack;

#[test]
fn static_component_attribute_values_are_type_checked() {
    check_pack(
        "static-component-prop-values-3197",
        "fix_history_cases::static_component_prop_values_3197::static_component_attribute_values_are_type_checked",
        "c5af134e4924e31f31c8aba74f9984e356e1f6c2",
        Some(3197),
        &[
            "static-string-prop",
            "static-string-prop-number-type",
            "static-escaped-literal-prop",
            "static-escaped-literal-prop-mismatch",
            "valueless-boolean-prop",
        ],
    );
}
