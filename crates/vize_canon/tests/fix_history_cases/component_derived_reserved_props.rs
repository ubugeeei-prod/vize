use crate::support::fix_history_runner::check_pack;

#[test]
fn component_derived_reserved_prop_shorthand() {
    check_pack(
        "component-derived-reserved-props",
        "fix_history_cases::component_derived_reserved_props::component_derived_reserved_prop_shorthand",
        "8fe6bd909063488f53b8c46e0a8301e57d1cd104",
        None,
        &[
            "reserved-prop-shorthand-from-component-props",
            "reserved-prop-number-value-type-error",
        ],
    );
}
