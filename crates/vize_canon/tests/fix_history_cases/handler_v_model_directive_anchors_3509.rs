use crate::support::fix_history_runner::check_pack;

#[test]
fn handler_and_v_model_errors_anchor_at_directive_names() {
    check_pack(
        "handler-v-model-directive-anchors-3509",
        "fix_history_cases::handler_v_model_directive_anchors_3509::handler_and_v_model_errors_anchor_at_directive_names",
        "4a18d23aebbcd4f6b6ccd332b23adee066b71fac",
        Some(3509),
        &[
            "component-event-handlers-valid",
            "component-event-handler-mismatches",
            "v-model-bindings-valid",
            "v-model-binding-mismatches",
        ],
    );
}
