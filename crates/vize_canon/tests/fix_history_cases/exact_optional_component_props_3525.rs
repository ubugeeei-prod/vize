use crate::support::fix_history_runner::check_pack;

#[test]
fn exact_optional_property_types_apply_to_component_props() {
    check_pack(
        "exact-optional-component-props-3525",
        "fix_history_cases::exact_optional_component_props_3525::exact_optional_property_types_apply_to_component_props",
        "14a39c33651fa00aceeb62924b519319bc9492d7",
        Some(3525),
        &[
            "explicit-undefined-optional-prop-exact",
            "explicit-undefined-optional-prop-loose",
            "correct-usages-exact",
            "wrongly-typed-prop-reported-once-exact",
        ],
    );
}
