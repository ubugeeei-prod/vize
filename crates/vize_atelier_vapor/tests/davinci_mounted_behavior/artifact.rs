//! Native S3 generation shares the same mounted observer as the reference and
//! legacy routes, with independent expected DOM identities and event delivery.

use crate::mounted_trace_with_identity;
use serde_json::{Value, json};

#[test]
fn native_artifact_updates_nested_and_separated_nodes_without_replacing_dom() {
    let source = r#"<main data-id="root"><span>fixed</span><div data-id="panel" :title="state.title"><b data-id="label">{{ state.label }}</b></div><i>tail</i><button data-id="action" :disabled="locked" @click="save">{{ caption }}</button></main>"#;
    let context =
        json!({"state": {"title": "A", "label": "first"}, "caption": "Start", "locked": false});
    let steps = json!([
        {"patch": {"state": {"title": "B", "label": "second"}, "caption": "Next"}},
        {"click": "action"},
        {"patch": {"locked": true}},
        {"click": "action"},
        {"patch": {"locked": false, "state": {"title": "雪", "label": "<ready>"}}},
        {"click": "action"}
    ]);
    let expected = json!([
        snapshot("A", "first", "Start", false, json!([])),
        snapshot("B", "second", "Next", false, json!([])),
        snapshot("B", "second", "Next", false, json!(["save"])),
        snapshot("B", "second", "Next", true, json!(["save"])),
        snapshot("B", "second", "Next", true, json!(["save"])),
        snapshot("雪", "<ready>", "Next", false, json!(["save"])),
        snapshot("雪", "<ready>", "Next", false, json!(["save", "save"])),
        {"tree": [], "events": ["save", "save"], "identities": []}
    ]);
    for backend in ["vdom", "vapor"] {
        let actual = mounted_trace_with_identity(
            backend,
            source,
            context.clone(),
            steps.clone(),
            false,
            true,
        );
        assert_eq!(
            actual, expected,
            "{backend} must preserve all four identified elements and deliver exactly two clicks"
        );
    }
}

fn snapshot(title: &str, label: &str, caption: &str, disabled: bool, events: Value) -> Value {
    let attributes = if disabled {
        json!({"data-id": "action", "disabled": ""})
    } else {
        json!({"data-id": "action"})
    };
    json!({
        "tree": [{"tag": "main", "attributes": {"data-id": "root"}, "children": [
            {"tag": "span", "attributes": {}, "children": ["fixed"]},
            {"tag": "div", "attributes": {"data-id": "panel", "title": title}, "children": [
                {"tag": "b", "attributes": {"data-id": "label"}, "children": [label]}
            ]},
            {"tag": "i", "attributes": {}, "children": ["tail"]},
            {"tag": "button", "attributes": attributes, "children": [caption], "disabled": disabled}
        ]}],
        "events": events,
        "identities": [["root", 0], ["panel", 2], ["label", 3], ["action", 5]]
    })
}

#[test]
fn native_artifact_handles_text_offsets_and_literal_escaping() {
    let source = r#"<div title='say "hi"'><b>first</b><i>second</i>{{ state.label }}<span>'tail'</span></div>"#;
    let actual = crate::assert_backends(
        source,
        json!({"state": {"label": "A"}}),
        json!([{"patch": {"state": {"label": "雪"}}}]),
    );
    let observation = |label| {
        json!({
            "tree": [{"tag": "div", "attributes": {"title": "say \"hi\""}, "children": [
                {"tag": "b", "attributes": {}, "children": ["first"]},
                {"tag": "i", "attributes": {}, "children": ["second"]},
                label,
                {"tag": "span", "attributes": {}, "children": ["'tail'"]}
            ]}], "events": []
        })
    };
    assert_eq!(
        actual,
        json!([observation("A"), observation("雪"), {"tree": [], "events": []}])
    );
}
