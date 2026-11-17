use crate::support::fix_history_runner::check_pack;

#[test]
fn runtime_emits_listener_payload_and_fallthrough_dom_listener() {
    check_pack(
        "runtime-emits-listener-payload-1359",
        "fix_history_cases::runtime_emits_listener_payload_1359::runtime_emits_listener_payload_and_fallthrough_dom_listener",
        "19b878c60242fff5328202728c92fbd23ef76d7d",
        Some(1359),
        &[
            "spread-runtime-emits-compatible-listener",
            "spread-runtime-emits-wrong-listener-payload",
            "undeclared-component-dom-listener-standard-mode",
        ],
    );
}
