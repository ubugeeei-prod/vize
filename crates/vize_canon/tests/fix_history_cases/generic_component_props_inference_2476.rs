use crate::support::fix_history_runner::check_pack;

#[test]
fn generic_component_props_infer_from_template() {
    check_pack(
        "generic-component-props-inference-2476",
        "fix_history_cases::generic_component_props_inference_2476::generic_component_props_infer_from_template",
        "f4dbdb44c080b869630d8ad83d46a95fa356e124",
        Some(2476),
        &[
            "generic-type-from-template-prop",
            "generic-model-type-from-v-model",
            "generic-model-type-from-static-value-key",
            "generic-model-type-from-static-value-key-mismatch",
        ],
    );
}
