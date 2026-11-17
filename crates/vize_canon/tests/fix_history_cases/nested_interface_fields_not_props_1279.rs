use crate::support::fix_history_runner::check_pack;

#[test]
fn nested_interface_fields_are_not_template_props() {
    check_pack(
        "nested-interface-fields-not-props-1279",
        "fix_history_cases::nested_interface_fields_not_props_1279::nested_interface_fields_are_not_template_props",
        "8de44530eef34bd2658c52c5f53a2963e0980b1d",
        Some(1279),
        &[
            "top-level-props-and-nested-member-access",
            "nested-field-used-as-bare-template-name",
        ],
    );
}
