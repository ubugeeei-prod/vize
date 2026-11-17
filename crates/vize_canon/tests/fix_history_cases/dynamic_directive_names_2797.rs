use crate::support::fix_history_runner::check_pack;

#[test]
fn dynamic_directive_names_stay_expressions_not_contract_keys() {
    check_pack(
        "dynamic-directive-names-2797",
        "fix_history_cases::dynamic_directive_names_2797::dynamic_directive_names_stay_expressions_not_contract_keys",
        "ce3a4f7a9d4718fc8ebbbfbfee00a3cf0e4ff1da",
        Some(2797),
        &[
            "dynamic-prop-and-event-names-unchecked-as-keys",
            "static-prop-beside-dynamic-name-still-checked",
            "dynamic-prop-value-still-checked",
            "dynamic-event-name-expression-still-checked",
        ],
    );
}
