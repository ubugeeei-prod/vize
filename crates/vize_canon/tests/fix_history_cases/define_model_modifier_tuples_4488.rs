use crate::support::fix_history_runner::check_pack;

#[test]
fn define_model_modifier_tuple_types_are_preserved() {
    check_pack(
        "define-model-modifier-tuples-4488",
        "fix_history_cases::define_model_modifier_tuples_4488::define_model_modifier_tuple_types_are_preserved",
        "08db7132a3fb81a618e2111c055fbc285bd4b86c",
        Some(4488),
        &[
            "default-model-modifier-tuple-parent",
            "model-getter-setter-generics",
            "unknown-modifier-tuple-member",
            "unknown-default-model-modifier-in-parent",
            "named-model-modifier-parent",
            "unknown-named-model-modifier-in-parent",
        ],
    );
}
