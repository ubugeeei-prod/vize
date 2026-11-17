use crate::support::fix_history_runner::check_pack;

#[test]
fn sequence_callback_props_keep_contextual_parameter_types() {
    check_pack(
        "sequence-prop-expressions-3553",
        "fix_history_cases::sequence_prop_expressions_3553::sequence_callback_props_keep_contextual_parameter_types",
        "ec3ef8ce3ddf5853d3659a83d9b8f151fdbe3f1f",
        Some(3553),
        &[
            "non-generic-sequence-callback",
            "non-generic-sequence-callback-wrong-member",
            "generic-sequence-callback",
            "generic-sequence-callback-wrong-member",
        ],
    );
}
