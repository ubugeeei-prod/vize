use crate::support::fix_history_runner::check_pack;

#[test]
fn dynamic_component_props_use_resolved_setup_binding_type() {
    check_pack(
        "dynamic-component-setup-props-4494",
        "fix_history_cases::dynamic_component_setup_props_4494::dynamic_component_props_use_resolved_setup_binding_type",
        "67dc97e8ab5c0d96697e3b8e0f04aa1678ccc14f",
        Some(4494),
        &[
            "dynamic-component-correct-prop",
            "dynamic-component-wrong-prop-type",
        ],
    );
}
