//! Slot prop keys are reactive without changing the selector or DOM identity.

use serde_json::{Value, json};
use vize_atelier_core::walk_probe::WalkCounts;
use vize_atelier_vapor::{VaporCompilerOptions, compile_vapor};
use vize_carton::Allocator;

use super::trace::trace;

fn view(text: &str) -> Value {
    let tail = json!({"tag": "i", "attributes": {"data-id": "tail"}, "children": ["tail"]});
    let children = if text.is_empty() {
        json!([tail])
    } else {
        json!([text, tail])
    };
    json!({"tree": [{"tag": "main", "attributes": {"data-id": "root"}, "children": children}], "events": [], "identities": [["root", 0], ["tail", 1]]})
}

fn unmounted() -> Value {
    json!({"tree": [], "events": [], "identities": []})
}

fn assert_trace(source: &str, context: Value, steps: Value, slots: Value, expected: Vec<Value>) {
    let allocator = Allocator::new();
    let before = WalkCounts::snapshot();
    let compiled = compile_vapor(
        &allocator,
        source,
        VaporCompilerOptions {
            prefix_identifiers: true,
            ..Default::default()
        },
    );
    assert!(
        compiled.error_messages.is_empty(),
        "{source}: {:?}",
        compiled.error_messages
    );
    assert_eq!(
        WalkCounts::snapshot().since(before).total_walks(),
        0,
        "{source}: native S3"
    );
    let native = trace(
        "davinci-mounted-trace.mjs",
        json!({"backend": "vapor", "code": compiled.code, "context": context, "steps": steps, "slots": slots, "identities": true}),
    );
    let official = trace(
        "davinci-upstream-vapor-trace.mjs",
        json!({"source": source, "context": context, "steps": steps, "slots": slots}),
    );
    assert_eq!(native, expected, "{source}: native trace");
    assert_eq!(official, expected, "{source}: official trace");
}

#[test]
fn computed_slot_keys_restore_static_values_in_both_authored_orders() {
    for key in ["name", "keys[index]", "name.toLowerCase()"] {
        for static_first in [true, false] {
            let props = if static_first {
                format!(r#"title="fixed" :[{key}]="value""#)
            } else {
                format!(r#":[{key}]="value" title="fixed""#)
            };
            let source =
                format!(r#"<main data-id="root"><slot {props}/><i data-id="tail">tail</i></main>"#);
            assert_trace(
                &source,
                json!({"name": "title", "keys": ["title", "other"], "index": 0, "value": "A"}),
                json!([{"patch": {"name": "other", "index": 1, "value": "B"}}, {"patch": {"name": "title", "index": 0, "value": "C"}}, {"patch": {"value": null}}]),
                json!({"default": {"prop": "title"}}),
                vec![
                    view(if static_first { "A" } else { "fixed" }),
                    view("fixed"),
                    view(if static_first { "C" } else { "fixed" }),
                    view(if static_first { "" } else { "fixed" }),
                    unmounted(),
                ],
            );
        }
    }
}

#[test]
fn computed_slot_key_spelling_stays_distinct_from_static_camelized_names() {
    for static_first in [true, false] {
        let props = if static_first {
            r#"data-id="fixed" :[name]="value""#
        } else {
            r#":[name]="value" data-id="fixed""#
        };
        let source =
            format!(r#"<main data-id="root"><slot {props}/><i data-id="tail">tail</i></main>"#);
        assert_trace(
            &source,
            json!({"name": "data-id", "value": "A"}),
            json!([{"patch": {"name": "dataId", "value": "B"}}, {"patch": {"name": "other", "value": "C"}}, {"patch": {"name": "data-id", "value": "D"}}]),
            json!({"default": {"prop": "dataId"}}),
            vec![
                view("fixed"),
                view(if static_first { "B" } else { "fixed" }),
                view("fixed"),
                view("fixed"),
                unmounted(),
            ],
        );
    }
}

#[test]
fn computed_slot_class_and_style_sources_preserve_raw_arrays_and_restore_values() {
    let displayed = |value: Value| serde_json::to_string_pretty(&value).unwrap();
    assert_trace(
        r#"<main data-id="root"><slot class="base" :class="classes" :[name]="value"/><i data-id="tail">tail</i></main>"#,
        json!({"name": "class", "value": "computed", "classes": ["active"]}),
        json!([{"patch": {"name": "other", "value": "B"}}, {"patch": {"name": "class", "value": ["next"], "classes": ["updated"]}}]),
        json!({"default": {"prop": "class"}}),
        vec![
            view(&displayed(json!([["base", ["active"]], "computed"]))),
            view(&displayed(json!([["base", ["active"]]]))),
            view(&displayed(json!([["base", ["updated"]], ["next"]]))),
            unmounted(),
        ],
    );
    assert_trace(
        r#"<main data-id="root"><slot style="color:red" :style="styles" :[name]="value"/><i data-id="tail">tail</i></main>"#,
        json!({"name": "style", "value": {"color": "blue"}, "styles": {"fontWeight": "bold"}}),
        json!([{"patch": {"name": "other", "value": "B"}}, {"patch": {"name": "style", "value": {"color": "green"}, "styles": {"fontSize": "12px"}}}]),
        json!({"default": {"prop": "style"}}),
        vec![
            view(&displayed(
                json!([["color:red", {"fontWeight": "bold"}], {"color": "blue"}]),
            )),
            view(&displayed(json!([["color:red", {"fontWeight": "bold"}]]))),
            view(&displayed(
                json!([["color:red", {"fontSize": "12px"}], {"color": "green"}]),
            )),
            unmounted(),
        ],
    );
}

#[test]
fn ordinary_slot_props_camelize_static_keys_and_merge_raw_class_style_values() {
    let displayed = |value: Value| serde_json::to_string_pretty(&value).unwrap();
    for (prop, authored, context, steps, expected) in [
        (
            "dataId",
            r#"data-id="fixed" disabled"#,
            json!({}),
            json!([{"patch": {}}]),
            vec![view("fixed"), view("fixed"), unmounted()],
        ),
        (
            "disabled",
            "disabled",
            json!({}),
            json!([{"patch": {}}]),
            vec![view(""), view(""), unmounted()],
        ),
        (
            "class",
            r#"class="base" :class="classes""#,
            json!({"classes": ["active"]}),
            json!([{"patch": {"classes": ["updated"]}}]),
            vec![
                view(&displayed(json!([["base", ["active"]]]))),
                view(&displayed(json!([["base", ["updated"]]]))),
                unmounted(),
            ],
        ),
        (
            "class",
            r#"class :class="classes""#,
            json!({"classes": ["active"]}),
            json!([{"patch": {"classes": ["updated"]}}]),
            vec![
                view(&displayed(json!([["active"]]))),
                view(&displayed(json!([["updated"]]))),
                unmounted(),
            ],
        ),
        (
            "style",
            r#"style="color:red" :style="styles""#,
            json!({"styles": {"fontWeight": "bold"}}),
            json!([{"patch": {"styles": {"color": "blue"}}}]),
            vec![
                view(&displayed(json!([["color:red", {"fontWeight": "bold"}]]))),
                view(&displayed(json!([["color:red", {"color": "blue"}]]))),
                unmounted(),
            ],
        ),
    ] {
        let source =
            format!(r#"<main data-id="root"><slot {authored}/><i data-id="tail">tail</i></main>"#);
        assert_trace(
            &source,
            context,
            steps,
            json!({"default": {"prop": prop}}),
            expected,
        );
    }
}

#[test]
fn computed_name_prop_does_not_steal_dynamic_slot_selection_or_fallback_lifetime() {
    let fallback_view = |label: &str| json!({"tree": [{"tag": "main", "attributes": {"data-id": "root"}, "children": [{"tag": "b", "attributes": {"data-id": "fallback"}, "children": [label]}, {"tag": "i", "attributes": {"data-id": "tail"}, "children": ["tail"]}]}], "events": [], "identities": [["root", 0], ["fallback", 2], ["tail", 1]]});
    assert_trace(
        r#"<main data-id="root"><slot :name="selected" :[name]="value"><b data-id="fallback">{{ fallback }}</b></slot><i data-id="tail">tail</i></main>"#,
        json!({"selected": "one", "name": "title", "value": "A", "fallback": "F0"}),
        json!([{"patch": {"selected": "two", "value": "B"}}, {"patch": {"selected": "absent", "fallback": "F1"}}, {"patch": {"name": "other", "value": "C", "fallback": "F2"}}, {"patch": {"selected": "one", "name": "title", "value": "D"}}]),
        json!({"one": {"prop": "title"}, "two": {"prop": "title"}}),
        vec![
            view("A"),
            view("B"),
            fallback_view("F1"),
            fallback_view("F2"),
            view("D"),
            unmounted(),
        ],
    );
}
