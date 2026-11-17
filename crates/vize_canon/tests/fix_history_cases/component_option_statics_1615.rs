use crate::support::fix_history_runner::check_pack;

#[test]
fn virtual_component_satisfies_concrete_component_constraint() {
    check_pack(
        "component-option-statics-1615",
        "fix_history_cases::component_option_statics_1615::virtual_component_satisfies_concrete_component_constraint",
        "01ae57c12f09b1bbe0be77097d5cd3788f3e65ad",
        Some(1615),
        &[
            "storybook-like-meta-component",
            "storybook-like-meta-wrong-component",
        ],
    );
}
