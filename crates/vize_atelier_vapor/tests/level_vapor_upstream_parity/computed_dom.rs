//! Computed DOM keys restore earlier sources and retain browser node identity.

use super::trace::assert_native_upstream_trace;
use serde_json::{Value, json};

fn view(attributes: Value) -> Value {
    json!({
        "tree": [{"tag": "main", "attributes": {"data-id": "root"}, "children": [
            {"tag": "div", "attributes": attributes, "children": ["box"]},
            {"tag": "i", "attributes": {"data-id": "tail"}, "children": ["tail"]},
        ]}],
        "events": [], "identities": [["root", 0], ["box", 1], ["tail", 2]],
    })
}

fn unmounted() -> Value {
    json!({"tree": [], "events": [], "identities": []})
}

#[test]
fn computed_dom_keys_restore_static_attributes_in_both_authored_orders() {
    for key in ["name", "keys[index]", "name.toLowerCase()", "title"] {
        for static_first in [true, false] {
            let props = if static_first {
                format!(r#"title="fixed" :[{key}]="value""#)
            } else {
                format!(r#":[{key}]="value" title="fixed""#)
            };
            let source = format!(
                r#"<main data-id="root"><div data-id="box" {props}>box</div><i data-id="tail">tail</i></main>"#
            );
            assert_native_upstream_trace(
                &source,
                json!({"name": "title", "title": "title", "keys": ["title", "data-role"], "index": 0, "value": "A"}),
                json!([
                    {"patch": {"name": "data-role", "title": "data-role", "index": 1, "value": "B"}},
                    {"patch": {"name": "title", "title": "title", "index": 0, "value": "C"}},
                    {"patch": {"value": null}},
                ]),
                vec![
                    view(json!({"data-id": "box", "title": if static_first {"A"} else {"fixed"}})),
                    view(json!({"data-id": "box", "data-role": "B", "title": "fixed"})),
                    view(json!({"data-id": "box", "title": if static_first {"C"} else {"fixed"}})),
                    view(if static_first {
                        json!({"data-id": "box"})
                    } else {
                        json!({"data-id": "box", "title": "fixed"})
                    }),
                    unmounted(),
                ],
            );
        }
    }
}

#[test]
fn computed_dom_class_and_style_collisions_restore_static_merges() {
    let source = r#"<main data-id="root"><div data-id="box" class="base" :class="classes" style="color:red" :style="styles" :[keys[index]]="value">box</div><i data-id="tail">tail</i></main>"#;
    assert_native_upstream_trace(
        source,
        json!({"keys": ["class", "style", "data-role"], "index": 0, "value": "computed", "classes": {"active": true}, "styles": {"backgroundColor": "yellow"}}),
        json!([
            {"patch": {"index": 1, "value": {"color": "blue"}}},
            {"patch": {"index": 2, "value": "B", "classes": ["other"], "styles": {"backgroundColor": "orange"}}},
            {"patch": {"index": 0, "value": null, "classes": [], "styles": {}}},
        ]),
        vec![
            view(
                json!({"class": "computed", "data-id": "box", "style": "color: red; background-color: yellow;"}),
            ),
            view(json!({"class": "base active", "data-id": "box", "style": "color: blue;"})),
            view(
                json!({"class": "base other", "data-id": "box", "data-role": "B", "style": "color: red; background-color: orange;"}),
            ),
            view(json!({"class": "", "data-id": "box", "style": "color: red;"})),
            unmounted(),
        ],
    );
}

#[test]
fn computed_dom_sources_merge_across_objects_in_authored_order() {
    let source = r#"<main data-id="root"><div data-id="box" class="base" :[name]="value" v-bind="attrs" title="fixed">box</div><i data-id="tail">tail</i></main>"#;
    assert_native_upstream_trace(
        source,
        json!({"name": "class", "value": "computed", "attrs": {"class": "object", "title": "ignored"}}),
        json!([
            {"patch": {"name": "title", "value": "ignored", "attrs": {"class": "next", "data-role": "B"}}},
            {"patch": {"name": "data-role", "value": "C", "attrs": {}}},
        ]),
        vec![
            view(json!({"data-id": "box", "class": "computed object", "title": "fixed"})),
            view(
                json!({"data-id": "box", "class": "base next", "data-role": "B", "title": "fixed"}),
            ),
            view(json!({"data-id": "box", "class": "base", "data-role": "C", "title": "fixed"})),
            unmounted(),
        ],
    );
}

#[test]
fn computed_input_value_restores_static_dom_property_on_rename() {
    let input_view = |value: &str, title: Option<&str>| {
        let attributes = if let Some(title) = title {
            json!({"data-id": "box", "value": value, "title": title})
        } else {
            json!({"data-id": "box", "value": value})
        };
        json!({
            "tree": [{"tag": "main", "attributes": {"data-id": "root"}, "children": [
                {"tag": "input", "attributes": attributes, "children": [], "value": value, "checked": false},
                {"tag": "i", "attributes": {"data-id": "tail"}, "children": ["tail"]},
            ]}], "events": [], "identities": [["root", 0], ["box", 1], ["tail", 2]],
        })
    };
    assert_native_upstream_trace(
        r#"<main data-id="root"><input data-id="box" value="fixed" :[name]="value"><i data-id="tail">tail</i></main>"#,
        json!({"name": "value", "value": "A"}),
        json!([
            {"patch": {"name": "title", "value": "B"}},
            {"patch": {"name": "value", "value": "C"}},
            {"patch": {"name": "title", "value": "D"}},
        ]),
        vec![
            input_view("A", None),
            input_view("fixed", Some("B")),
            input_view("C", None),
            input_view("fixed", Some("D")),
            unmounted(),
        ],
    );
}

#[test]
fn computed_dom_loop_aliases_restore_props_through_keyed_reorder_and_removal() {
    let item = |id: &str, attributes: Value| json!({"tag": "li", "attributes": attributes, "children": [id]});
    let loop_view = |children: Value, identities: Value| {
        json!({
            "tree": [{"tag": "ul", "attributes": {"data-id": "root"}, "children": children}],
            "events": [], "identities": identities,
        })
    };
    assert_native_upstream_trace(
        r#"<ul data-id="root"><li v-for="item in items" :key="item.id" :data-id="item.id" title="fixed" :[item.name]="item.value">{{ item.label }}</li></ul>"#,
        json!({"items": [{"id": "first", "name": "title", "value": "A", "label": "first"}, {"id": "second", "name": "title", "value": "B", "label": "second"}]}),
        json!([
            {"patch": {"items": [{"id": "second", "name": "data-role", "value": "C", "label": "second"}, {"id": "first", "name": "data-role", "value": "D", "label": "first"}]}},
            {"patch": {"items": [{"id": "second", "name": "title", "value": "E", "label": "second"}]}},
        ]),
        vec![
            loop_view(
                json!([
                    item("first", json!({"data-id": "first", "title": "A"})),
                    item("second", json!({"data-id": "second", "title": "B"}))
                ]),
                json!([["root", 0], ["first", 1], ["second", 2]]),
            ),
            loop_view(
                json!([
                    item(
                        "second",
                        json!({"data-id": "second", "data-role": "C", "title": "fixed"})
                    ),
                    item(
                        "first",
                        json!({"data-id": "first", "data-role": "D", "title": "fixed"})
                    )
                ]),
                json!([["root", 0], ["second", 2], ["first", 1]]),
            ),
            loop_view(
                json!([item("second", json!({"data-id": "second", "title": "E"}))]),
                json!([["root", 0], ["second", 2]]),
            ),
            unmounted(),
        ],
    );
}
