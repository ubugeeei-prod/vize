use super::{completion, drain};
use corsa::runtime::broadcast;
use corsa_lsp::jsonrpc::InboundEvent;
use serde_json::{Value, json};

fn event(method: &str, params: Value) -> InboundEvent {
    InboundEvent::Notification {
        method: method.into(),
        params,
    }
}

#[test]
fn complete_provider_envelopes_and_failures_match_pinned_packets() {
    let cases: Vec<Value> = serde_json::from_str(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../tests/_fixtures/differential/typechecker/native-shutdown-completion-10524/completion-envelopes.json"))).unwrap();
    for case in cases {
        let input = case.get("input").unwrap();
        let actual = match completion(input) {
            Ok(true) => json!({"kind":"complete"}),
            Ok(false) => json!({"kind":"unrelated"}),
            Err(error) => json!({"kind":"malformed","error":error.to_string()}),
        };
        assert_eq!(&actual, case.get("expected").unwrap(), "{input}");
    }
}

#[test]
fn fresh_subscription_ignores_past_completion_and_retains_unrelated_wire_values() {
    let (sender, historical) = broadcast();
    let stale = json!({"type":3,"message":"handled method 'shutdown' (99) in 1ns"});
    assert_eq!(sender.send(event("window/logMessage", stale.clone())), 1);
    let current = sender.subscribe();
    let ignored = [
        (
            "window/showMessage",
            json!({"type":3,"message":"handled method 'shutdown' (7) in 1ns"}),
        ),
        (
            "window/logMessage",
            json!({"type":3,"message":"handled method 'textDocument/hover' (6) in 1ns"}),
        ),
        (
            "window/logMessage",
            json!({"type":2,"message":"handled method 'shutdown' (7) in 1ns"}),
        ),
        (
            "window/logMessage",
            json!({"type":"3","message":"handled method 'shutdown' (7) in 1ns"}),
        ),
    ];
    for (method, params) in &ignored {
        assert_eq!(sender.send(event(method, params.clone())), 2);
    }
    let completed = json!({"type":3,"message":"handled method 'shutdown' (7) in 6.071µs"});
    assert_eq!(
        sender.send(event("window/logMessage", completed.clone())),
        2
    );
    drain(&current).unwrap();
    // Observing the completion must not consume or rewrite another subscriber's
    // stream, including the stale notification outside this subscription.
    let expected = std::iter::once(("window/logMessage", stale))
        .chain(ignored)
        .chain(std::iter::once(("window/logMessage", completed)));
    for (expected_method, expected_params) in expected {
        let InboundEvent::Notification { method, params } = historical.recv().unwrap() else {
            panic!("complete notification expected")
        };
        assert_eq!(
            (method.as_str(), params),
            (expected_method, expected_params)
        );
    }
}

#[test]
fn unrelated_fresh_envelopes_cannot_complete_shutdown_without_current_info() {
    for (method, params) in [
        (
            "window/showMessage",
            json!({"type":3,"message":"handled method 'shutdown' (7) in 1ns"}),
        ),
        (
            "window/logMessage",
            json!({"type":3,"message":"handled method 'textDocument/hover' (6) in 1ns"}),
        ),
        (
            "window/logMessage",
            json!({"type":2,"message":"handled method 'shutdown' (7) in 1ns"}),
        ),
        (
            "window/logMessage",
            json!({"type":"3","message":"handled method 'shutdown' (7) in 1ns"}),
        ),
    ] {
        let (sender, events) = broadcast();
        assert_eq!(sender.send(event(method, params.clone())), 1);
        drop(sender);
        assert_eq!(
            drain(&events).unwrap_err().to_string(),
            "process is closed: native shutdown completion",
            "{method}: {params}"
        );
    }
}

#[test]
fn disconnected_completion_channel_is_not_a_successful_drain() {
    let (sender, events) = broadcast::<InboundEvent>();
    drop(sender);
    assert_eq!(
        drain(&events).unwrap_err().to_string(),
        "process is closed: native shutdown completion"
    );
}
