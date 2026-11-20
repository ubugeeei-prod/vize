use crate::support::fix_history_runner::check_pack;

#[test]
fn component_event_handlers_use_child_emit_payloads() {
    check_pack(
        "component-emit-handler-payload-855",
        "fix_history_cases::component_emit_handler_payload_855::component_event_handlers_use_child_emit_payloads",
        "46e967b5b5b1c6cdbb903fa5c85cd42c8f0d058a",
        Some(855),
        &[
            "zero-arg-event-accepts-defaulted-handler",
            "tuple-emit-payload-matching-handler",
            "tuple-emit-payload-wrong-handler",
            "overload-emit-payload-matching-handler",
            "overload-emit-payload-wrong-handler",
        ],
    );
}
