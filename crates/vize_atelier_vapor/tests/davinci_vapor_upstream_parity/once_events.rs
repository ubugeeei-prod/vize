//! Mount-only rendering retains live handlers without refreshing DOM values.

use serde_json::{Value, json};

use super::trace::assert_native_upstream_trace;

fn view(live: &str, events: Value) -> Value {
    json!({
        "tree": [{"tag": "main", "attributes": {"data-id": "root"}, "children": [
            {"tag": "button", "attributes": {"data-id": "button", "title": "A"}, "children": ["A"], "disabled": false},
            {"tag": "span", "attributes": {"data-id": "live"}, "children": [live]},
        ]}],
        "events": events,
        "identities": [["root", 0], ["button", 1], ["live", 2]],
    })
}

#[test]
fn frozen_values_and_live_inline_handlers_match_official_vapor() {
    for (owner, event) in [
        (
            "<button v-once data-id=\"button\" :title=\"label\" @click=\"record(label)\">{{ label }}</button>",
            "click",
        ),
        (
            "<button v-once data-id=\"button\" :title=\"label\" @click.stop.prevent=\"record(label)\">{{ label }}</button>",
            "click",
        ),
        (
            "<button v-once data-id=\"button\" :title=\"label\" @keydown.enter.stop.prevent=\"record(label)\">{{ label }}</button>",
            "keydown",
        ),
    ] {
        let source = format!(
            "<main data-id=\"root\">{owner}<span data-id=\"live\">{{{{ label }}}}</span></main>"
        );
        let event_step = json!({"event": event, "selector": "button", "key": "Enter"});
        assert_native_upstream_trace(
            &source,
            json!({"label": "A"}),
            json!([event_step.clone(), {"patch": {"label": "B"}}, event_step.clone(), {"patch": {"label": "C"}}, event_step]),
            vec![
                view("A", json!([])),
                view("A", json!(["A"])),
                view("B", json!(["A"])),
                view("B", json!(["A", "B"])),
                view("C", json!(["A", "B"])),
                view("C", json!(["A", "B", "C"])),
                json!({"tree": [], "events": ["A", "B", "C"], "identities": []}),
            ],
        );
    }
}

#[test]
fn once_listener_is_not_reinstalled_when_state_changes() {
    assert_native_upstream_trace(
        r#"<main data-id="root"><button v-once data-id="button" :title="label" @click.once="save">{{ label }}</button><span data-id="live">{{ label }}</span></main>"#,
        json!({"label": "A"}),
        json!([{"click": "button"}, {"patch": {"label": "B"}}, {"click": "button"}, {"patch": {"label": "C"}}, {"click": "button"}]),
        vec![
            view("A", json!([])),
            view("A", json!(["save"])),
            view("B", json!(["save"])),
            view("B", json!(["save"])),
            view("C", json!(["save"])),
            view("C", json!(["save"])),
            json!({"tree": [], "events": ["save"], "identities": []}),
        ],
    );
}

#[test]
fn inherited_once_keeps_nested_event_handlers_live() {
    let expected_view = |live, events| {
        json!({
            "tree": [{"tag": "main", "attributes": {"data-id": "root"}, "children": [
                {"tag": "div", "attributes": {"data-id": "frozen", "title": "A"}, "children": [
                    {"tag": "button", "attributes": {"data-id": "button"}, "children": ["A"], "disabled": false},
                ]},
                {"tag": "span", "attributes": {"data-id": "live"}, "children": [live]},
            ]}],
            "events": events,
            "identities": [["root", 0], ["frozen", 1], ["button", 2], ["live", 3]],
        })
    };
    assert_native_upstream_trace(
        r#"<main data-id="root"><div v-once data-id="frozen" :title="label"><button data-id="button" @click.stop.prevent="record(label)">{{ label }}</button></div><span data-id="live">{{ label }}</span></main>"#,
        json!({"label": "A"}),
        json!([{"click": "button"}, {"patch": {"label": "B"}}, {"click": "button"}]),
        vec![
            expected_view("A", json!([])),
            expected_view("A", json!(["A"])),
            expected_view("B", json!(["A"])),
            expected_view("B", json!(["A", "B"])),
            json!({"tree": [], "events": ["A", "B"], "identities": []}),
        ],
    );
}
