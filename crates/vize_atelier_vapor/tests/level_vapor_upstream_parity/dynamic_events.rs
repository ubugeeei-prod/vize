//! Reactive event names, modifier guards and listener cleanup use the actual
//! Vue runtime with both compilers. Node identities stay stable across rebinding.

use serde_json::{Value, json};

use super::trace::assert_native_upstream_trace;

fn view(events: &[&str]) -> Value {
    json!({
        "tree": [{"tag": "main", "attributes": {"data-id": "root"}, "children": [
            {"tag": "button", "attributes": {"data-id": "button"}, "children": ["go"], "disabled": false},
            {"tag": "i", "attributes": {"data-id": "tail"}, "children": ["tail"]},
        ]}],
        "events": events,
        "identities": [["root", 0], ["button", 1], ["tail", 2]],
    })
}

fn unmounted(events: &[&str]) -> Value {
    json!({"tree": [], "events": events, "identities": []})
}

fn source(name: &str, modifiers: &str) -> String {
    r#"<main data-id="root"><button data-id="button" @[NAME]MODS="save">go</button><i data-id="tail">tail</i></main>"#
        .replace("NAME", name)
        .replace("MODS", modifiers)
}

#[test]
fn computed_names_rebind_without_duplicate_or_stale_listeners() {
    let context = json!({"eventName": "click", "selected": "one", "enabled": true,
        "first": "click", "second": "mousedown", "names": {"one": "click", "two": "mousedown"}});
    for (name, patch) in [
        ("eventName", json!({"eventName": "mousedown"})),
        ("names[selected]", json!({"selected": "two"})),
        ("enabled?first:second", json!({"enabled": false})),
        ("eventName.toLowerCase()", json!({"eventName": "MOUSEDOWN"})),
    ] {
        let steps = json!([
            {"event": "click", "selector": "[data-id=button]"},
            {"patch": patch},
            {"event": "click", "selector": "[data-id=button]"},
            {"event": "mousedown", "selector": "[data-id=button]"},
            {"patch": patch},
            {"event": "mousedown", "selector": "[data-id=button]"},
            {"patch": context},
            {"event": "mousedown", "selector": "[data-id=button]"},
            {"event": "click", "selector": "[data-id=button]"},
        ]);
        let mut expected = [0, 1, 1, 1, 2, 2, 3, 3, 3, 4]
            .map(|count| view(&vec!["save"; count]))
            .to_vec();
        expected.push(unmounted(&["save"; 4]));
        assert_native_upstream_trace(&source(name, ""), context.clone(), steps, expected);
    }
}

#[test]
fn computed_once_options_reset_only_when_event_name_changes() {
    let steps = json!([
        {"event": "click", "selector": "[data-id=button]"},
        {"event": "click", "selector": "[data-id=button]"},
        {"patch": {"eventName": "click"}},
        {"event": "click", "selector": "[data-id=button]"},
        {"patch": {"eventName": "mousedown"}},
        {"event": "click", "selector": "[data-id=button]"},
        {"event": "mousedown", "selector": "[data-id=button]"},
        {"event": "mousedown", "selector": "[data-id=button]"},
    ]);
    let mut expected = [0, 1, 1, 1, 1, 1, 1, 2, 2]
        .map(|count| view(&vec!["save"; count]))
        .to_vec();
    expected.push(unmounted(&["save"; 2]));
    assert_native_upstream_trace(
        &source("eventName", ".once.capture.passive"),
        json!({"eventName": "click"}),
        steps,
        expected,
    );
}

#[test]
fn computed_keyboard_modifiers_keep_guards_after_rebinding() {
    let steps = json!([
        {"event": "keyup", "selector": "[data-id=button]", "key": "Escape"},
        {"event": "keyup", "selector": "[data-id=button]", "key": "Enter"},
        {"patch": {"eventName": "keydown"}},
        {"event": "keyup", "selector": "[data-id=button]", "key": "Enter"},
        {"event": "keydown", "selector": "[data-id=button]", "key": "Escape"},
        {"event": "keydown", "selector": "[data-id=button]", "key": "Enter"},
    ]);
    let mut expected = [0, 0, 1, 1, 1, 1, 2]
        .map(|count| view(&vec!["save"; count]))
        .to_vec();
    expected.push(unmounted(&["save"; 2]));
    assert_native_upstream_trace(
        &source("eventName", ".enter.stop.prevent"),
        json!({"eventName": "keyup"}),
        steps,
        expected,
    );
}
