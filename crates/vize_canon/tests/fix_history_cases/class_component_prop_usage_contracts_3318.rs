use crate::support::fix_history_runner::check_pack;

#[test]
fn class_component_prop_decorators_enforce_usage_site_contracts() {
    check_pack(
        "class-component-prop-usage-contracts-3318",
        "fix_history_cases::class_component_prop_usage_contracts_3318::class_component_prop_decorators_enforce_usage_site_contracts",
        "519cec5990d37434620c3ac38e3750a0eefa195b",
        Some(3318),
        &[
            "hello-decorator-required-prop-passed",
            "hello-decorator-missing-and-mismatched-prop",
            "optional-default-and-annotated-props-accepted",
            "optional-default-and-annotated-props-mismatched",
            "renaming-decorators-resolve-through-instance",
            "renaming-decorator-member-wrong-use",
        ],
    );
}
