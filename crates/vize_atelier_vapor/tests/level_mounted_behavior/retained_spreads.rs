//! Ordered object sources must agree across the retained and native lanes.

use super::runtime::mounted_trace_with_identity;
use serde_json::{Value, json};

fn view(attributes: Value) -> Value {
    json!({
        "tree": [{"tag": "div", "attributes": attributes, "children": []}],
        "events": [], "identities": [["box", 0]],
    })
}

fn assert_lanes(source: &str, context: Value, steps: Value, expected: Value) {
    for backend in ["vdom", "vapor", "vapor-legacy"] {
        let actual = mounted_trace_with_identity(
            backend,
            source,
            context.clone(),
            steps.clone(),
            false,
            true,
        );
        assert_eq!(actual, expected, "{backend}: {source}");
    }
}

#[test]
fn object_sources_preserve_and_restore_both_static_authored_orders() {
    for static_first in [false, true] {
        let source = if static_first {
            r#"<div data-id="box" title="fixed" v-bind="attrs"></div>"#
        } else {
            r#"<div data-id="box" v-bind="attrs" title="fixed"></div>"#
        };
        assert_lanes(
            source,
            json!({"attrs": {"title": "first", "data-role": "A"}}),
            json!([
                {"patch": {"attrs": {"title": "second", "data-role": "B"}}},
                {"patch": {"attrs": {}}},
                {"patch": {"attrs": {"title": "last"}}},
            ]),
            json!([
                view(json!({"data-id": "box", "title": if static_first {"first"} else {"fixed"}, "data-role": "A"})),
                view(json!({"data-id": "box", "title": if static_first {"second"} else {"fixed"}, "data-role": "B"})),
                view(json!({"data-id": "box", "title": "fixed"})),
                view(json!({"data-id": "box", "title": if static_first {"last"} else {"fixed"}})),
                {"tree": [], "events": [], "identities": []},
            ]),
        );
    }
}

#[test]
fn multiple_objects_and_bound_sources_publish_one_combined_prop_state() {
    assert_lanes(
        r#"<div data-id="box" v-bind="first" :title="title" v-bind="last"></div>"#,
        json!({"first": {"title": "first", "data-role": "kept"}, "title": "middle", "last": {"title": "last", "data-last": "A"}}),
        json!([
            {"patch": {"last": {}}},
            {"patch": {"title": "updated"}},
            {"patch": {"first": {}, "last": {"title": "override"}}},
            {"patch": {"last": {}}},
        ]),
        json!([
            view(json!({"data-id": "box", "title": "last", "data-role": "kept", "data-last": "A"})),
            view(json!({"data-id": "box", "title": "middle", "data-role": "kept"})),
            view(json!({"data-id": "box", "title": "updated", "data-role": "kept"})),
            view(json!({"data-id": "box", "title": "override"})),
            view(json!({"data-id": "box", "title": "updated"})),
            {"tree": [], "events": [], "identities": []},
        ]),
    );
}
