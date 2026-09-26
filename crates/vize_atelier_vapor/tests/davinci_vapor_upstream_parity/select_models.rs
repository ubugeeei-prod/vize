//! Select models compare browser-visible selection and two-way state updates.

use serde_json::{Value, json};

use super::trace::assert_native_upstream_trace;

fn view(values: [&str; 2], selected: [bool; 2], mirror: &str, multiple: bool) -> Value {
    let attributes = if multiple {
        json!({"data-id": "select", "multiple": ""})
    } else {
        json!({"data-id": "select"})
    };
    let value = values
        .into_iter()
        .zip(selected)
        .find_map(|(value, on)| on.then_some(value))
        .unwrap_or("");
    json!({
        "tree": [{"tag": "main", "attributes": {"data-id": "root"}, "children": [
            {"tag": "select", "attributes": attributes, "value": value, "children": [
                {"tag": "option", "attributes": {"value": values[0]}, "children": ["A"], "selected": selected[0]},
                {"tag": "option", "attributes": {"value": values[1]}, "children": ["B"], "selected": selected[1]},
            ]},
            {"tag": "span", "attributes": {"data-id": "mirror"}, "children": if mirror.is_empty() { vec![] } else { vec![mirror] }},
        ]}],
        "events": [],
        "identities": [["root", 0], ["select", 1], ["mirror", 4]],
    })
}

fn unmounted() -> Value {
    json!({"tree": [], "events": [], "identities": []})
}

#[test]
fn single_select_matches_official_user_changes_and_external_updates() {
    let source = r#"<main data-id="root"><select data-id="select" v-model="value"><option value="a">A</option><option value="b">B</option></select><span data-id="mirror">{{ value }}</span></main>"#;
    assert_native_upstream_trace(
        source,
        json!({"value": "a"}),
        json!([
            {"patch": {"value": "b"}},
            {"event": "change", "selector": "select", "selectedValues": ["a"]},
            {"patch": {"value": "missing"}},
            {"patch": {"value": "b"}},
        ]),
        vec![
            view(["a", "b"], [true, false], "a", false),
            view(["a", "b"], [false, true], "b", false),
            view(["a", "b"], [true, false], "a", false),
            view(["a", "b"], [false, false], "missing", false),
            view(["a", "b"], [false, true], "b", false),
            unmounted(),
        ],
    );
}

#[test]
fn multiple_select_matches_official_array_model_updates() {
    let source = r#"<main data-id="root"><select data-id="select" multiple v-model="value"><option value="a">A</option><option value="b">B</option></select><span data-id="mirror">{{ value.join(',') }}</span></main>"#;
    assert_native_upstream_trace(
        source,
        json!({"value": ["a"]}),
        json!([
            {"event": "change", "selector": "select", "selectedValues": ["a", "b"]},
            {"patch": {"value": ["b"]}},
            {"patch": {"value": []}},
            {"event": "change", "selector": "select", "selectedValues": ["a"]},
        ]),
        vec![
            view(["a", "b"], [true, false], "a", true),
            view(["a", "b"], [true, true], "a,b", true),
            view(["a", "b"], [false, true], "b", true),
            view(["a", "b"], [false, false], "", true),
            view(["a", "b"], [true, false], "a", true),
            unmounted(),
        ],
    );
}

#[test]
fn number_select_preserves_numeric_assignment_type() {
    let source = r#"<main data-id="root"><select data-id="select" v-model.number="value"><option value="1">A</option><option value="2">B</option></select><span data-id="mirror">{{ typeof value }}:{{ value }}</span></main>"#;
    assert_native_upstream_trace(
        source,
        json!({"value": 1}),
        json!([
            {"event": "change", "selector": "select", "selectedValues": ["2"]},
            {"patch": {"value": 1}},
        ]),
        vec![
            view(["1", "2"], [true, false], "number:1", false),
            view(["1", "2"], [false, true], "number:2", false),
            view(["1", "2"], [true, false], "number:1", false),
            unmounted(),
        ],
    );
}

#[test]
fn bound_object_options_preserve_model_identity_and_updates() {
    let source = r#"<main data-id="root"><select data-id="select" v-model="value"><option :value="first">A</option><option :value="second">B</option></select><span data-id="mirror">{{ value.id }}</span></main>"#;
    assert_native_upstream_trace(
        source,
        json!({"value": {"id": "a"}, "first": {"id": "a"}, "second": {"id": "b"}}),
        json!([
            {"event": "change", "selector": "select", "selectedIndexes": [1]},
            {"patch": {"first": {"id": "c"}, "value": {"id": "c"}}},
            {"event": "change", "selector": "select", "selectedIndexes": [1]},
        ]),
        vec![
            view(["[object Object]"; 2], [true, false], "a", false),
            view(["[object Object]"; 2], [false, true], "b", false),
            view(["[object Object]"; 2], [true, false], "c", false),
            view(["[object Object]"; 2], [false, true], "b", false),
            unmounted(),
        ],
    );
}
