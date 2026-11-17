use crate::support::fix_history_runner::check_pack;

#[test]
fn synthesized_emit_listeners_return_any() {
    check_pack(
        "synthesized-listener-any-return-3897",
        "fix_history_cases::synthesized_listener_any_return_3897::synthesized_emit_listeners_return_any",
        "29aeaa770fb27dc98e5a39e830aa977c91064c85",
        Some(3897),
        &[
            "event-handler-wrong-payload-and-arity",
            "event-handler-matching-payload",
            "generic-form-submit-listener-payload",
            "generic-form-submit-listener-valid",
        ],
    );
}
