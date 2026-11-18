use crate::support::fix_history_runner::check_pack;

#[test]
fn define_model_contributes_props_and_update_emits() {
    check_pack(
        "define-model-props-emits-1133",
        "fix_history_cases::define_model_props_emits_1133::define_model_contributes_props_and_update_emits",
        "4185649de7c5a859eedde74984d7eb253deae618",
        Some(1133),
        &[
            "model-component-template",
            "model-component-wrong-update-emit",
            "parent-models-and-emits",
            "parent-wrong-model-type",
            "parent-wrong-update-handler",
            "parent-missing-required-model",
        ],
    );
}
