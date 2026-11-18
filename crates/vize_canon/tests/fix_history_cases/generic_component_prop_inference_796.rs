use crate::support::fix_history_runner::check_pack;

#[test]
fn generic_component_props_infer_child_generic() {
    check_pack(
        "generic-component-prop-inference-796",
        "fix_history_cases::generic_component_prop_inference_796::generic_component_props_infer_child_generic",
        "f228424b649d224ce89d4f39ed0bc2fe09f8b225",
        Some(796),
        &[
            "well-typed-generic-component-props",
            "generic-component-prop-mismatch",
        ],
    );
}
