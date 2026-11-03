//! Deferred Teleport changes destinations without remounting its contents.

use serde_json::{Value, json};

use super::trace::{assert_native_upstream_trace, assert_native_upstream_trace_with_targets};

fn view(destination: &str, label: &str, count: usize) -> Value {
    let button = json!({"tag": "button", "attributes": {"data-id": "button"}, "children": [label], "disabled": false});
    let mut children = vec![
        json!({"tag": "div", "attributes": {"data-id": "a", "id": "target-a"}, "children": if destination == "a" { vec![button.clone()] } else { vec![] }}),
        json!({"tag": "div", "attributes": {"data-id": "b", "id": "target-b"}, "children": if destination == "b" { vec![button.clone()] } else { vec![] }}),
    ];
    if destination == "disabled" {
        children.push(button);
    }
    children.push(json!({"tag": "i", "attributes": {"data-id": "tail"}, "children": ["tail"]}));
    json!({
        "tree": [{"tag": "main", "attributes": {"data-id": "root"}, "children": children}],
        "events": vec!["save"; count],
        "identities": if destination == "a" {
            json!([["root", 0], ["a", 1], ["button", 2], ["b", 3], ["tail", 4]])
        } else {
            json!([["root", 0], ["a", 1], ["b", 3], ["button", 2], ["tail", 4]])
        },
    })
}

fn external_view(destination: &str, label: &str, count: usize) -> Value {
    let button = json!({"tag": "button", "attributes": {"data-id": "button"}, "children": [label], "disabled": false});
    let tail = json!({"tag": "i", "attributes": {"data-id": "tail"}, "children": ["tail"]});
    json!({
        "tree": [{"tag": "main", "attributes": {"data-id": "root"}, "children": if destination == "disabled" { vec![button.clone(), tail] } else { vec![tail] }}],
        "events": vec!["save"; count],
        "targets": {
            "outside-a": if destination == "a" { vec![button.clone()] } else { vec![] },
            "outside-b": if destination == "b" { vec![button] } else { vec![] },
        },
        "identities": if destination == "disabled" {
            json!([["root", 0], ["button", 2], ["tail", 1]])
        } else { json!([["root", 0], ["tail", 1], ["button", 2]]) },
    })
}

#[test]
fn external_destinations_preserve_events_and_are_empty_after_unmount() {
    assert_native_upstream_trace_with_targets(
        r#"<main data-id="root"><Teleport :to="target" :disabled="disabled"><button data-id="button" @click="save">{{ label }}</button></Teleport><i data-id="tail">tail</i></main>"#,
        json!({"target": "#outside-a", "disabled": false, "label": "A"}),
        json!([
            {"click": "button"},
            {"patch": {"target": "#outside-b", "label": "B"}},
            {"click": "button"},
            {"patch": {"disabled": true, "label": "C"}},
            {"click": "button"},
            {"patch": {"disabled": false}},
            {"click": "button"},
        ]),
        vec![
            external_view("a", "A", 0),
            external_view("a", "A", 1),
            external_view("b", "B", 1),
            external_view("b", "B", 2),
            external_view("disabled", "C", 2),
            external_view("disabled", "C", 3),
            external_view("b", "C", 3),
            external_view("b", "C", 4),
            json!({"tree": [], "events": vec!["save"; 4], "identities": [], "targets": {"outside-a": [], "outside-b": []}}),
        ],
        json!(["outside-a", "outside-b"]),
    );
}

#[test]
fn destinations_disabled_state_and_deferred_mount_match_official_vapor() {
    for (target, patch) in [
        ("target", json!({"target": "#target-b", "label": "B"})),
        ("targets[selected]", json!({"selected": "b", "label": "B"})),
        ("useA?first:second", json!({"useA": false, "label": "B"})),
    ] {
        let source = r#"<main data-id="root"><div id="target-a" data-id="a"></div><div id="target-b" data-id="b"></div><Teleport :to="TARGET" :disabled="disabled" defer><button data-id="button" @click="save">{{ label }}</button></Teleport><i data-id="tail">tail</i></main>"#.replace("TARGET", target);
        assert_native_upstream_trace(
            &source,
            json!({"target": "#target-a", "targets": {"a": "#target-a", "b": "#target-b"}, "selected": "a", "useA": true, "first": "#target-a", "second": "#target-b", "disabled": false, "label": "A"}),
            json!([
                {"click": "button"},
                {"patch": patch},
                {"click": "button"},
                {"patch": {"disabled": true, "label": "C"}},
                {"click": "button"},
                {"patch": {"disabled": false}},
                {"click": "button"},
            ]),
            vec![
                view("a", "A", 0),
                view("a", "A", 1),
                view("b", "B", 1),
                view("b", "B", 2),
                view("disabled", "C", 2),
                view("disabled", "C", 3),
                view("b", "C", 3),
                view("b", "C", 4),
                json!({"tree": [], "events": vec!["save"; 4], "identities": []}),
            ],
        );
    }
}
