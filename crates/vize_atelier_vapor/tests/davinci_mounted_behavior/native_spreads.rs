//! Native S3 `v-bind`/`v-on` objects through the published runtimes. Vue
//! merges an element's props in authored order: a later source wins and
//! `class`/`style` concatenate. Expectations are written out independently of
//! either compiler.

use super::native_control::element;
use super::runtime::{Child, mounted_trace_with_components};
use crate::mounted_trace_with_identity;
use serde_json::{Value, json};

#[test]
fn native_element_objects_merge_props_in_authored_order() {
    let source = r#"<main><div data-id="box" class="base" v-bind="attrs" title="fixed" :aria-label="label">{{ label }}</div></main>"#;
    let context =
        json!({"attrs": {"class": "x", "title": "from-attrs", "data-role": "a"}, "label": "L"});
    let steps = json!([
        {"patch": {"attrs": {"class": "y", "data-role": "b"}, "label": "M"}},
        {"patch": {"attrs": {"title": "late"}}}
    ]);
    let view = |attributes: Value, text: &str| {
        json!({
            "tree": [element("main", json!({}), json!([element("div", attributes, json!([text]))]))],
            "events": [],
            "identities": [["box", 1]]
        })
    };
    let unmounted = json!({"tree": [], "events": [], "identities": []});
    // The static title follows the object, so it wins over the object's.
    let expected = json!([
        view(
            json!({"data-id": "box", "class": "base x", "title": "fixed", "data-role": "a", "aria-label": "L"}),
            "L"
        ),
        view(
            json!({"data-id": "box", "class": "base y", "title": "fixed", "data-role": "b", "aria-label": "M"}),
            "M"
        ),
        view(
            json!({"data-id": "box", "class": "base", "title": "fixed", "aria-label": "M"}),
            "M"
        ),
        unmounted
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
        assert_eq!(actual, expected, "{backend}");
    }
    // The retained lane keeps static attributes in the template and applies
    // the object after them, so the object overrides the later static title
    // and replaces the static class (a P3-6 finding; the native lane fixes it).
    let legacy = json!([
        view(
            json!({"data-id": "box", "class": "x", "title": "from-attrs", "data-role": "a", "aria-label": "L"}),
            "L"
        ),
        view(
            json!({"data-id": "box", "class": "y", "data-role": "b", "aria-label": "M"}),
            "M"
        ),
        view(
            json!({"data-id": "box", "class": "", "title": "late", "aria-label": "M"}),
            "M"
        ),
        unmounted
    ]);
    let actual = mounted_trace_with_identity("vapor-legacy", source, context, steps, false, true);
    assert_eq!(actual, legacy, "vapor-legacy");
}

#[test]
fn native_component_objects_pass_props_and_listeners() {
    let tag: Child<'_> = (
        "Tag",
        r#"<button data-id="tag" :title="title" @click="$emit('go', label)">{{ label }}</button>"#,
        &["label", "title"],
        &["go"],
    );
    let view = |title: &str, events: Value| {
        json!({
            "tree": [element("main", json!({}), json!([
                {"tag": "button", "attributes": {"data-id": "tag", "title": title}, "disabled": false, "children": ["fixed"]},
                element("span", json!({"data-id": "out"}), json!([
                    {"tag": "button", "attributes": {"data-id": "own"}, "disabled": false, "children": ["own"]}
                ]))
            ]))],
            "events": events,
            "identities": [["tag", 1], ["out", 2], ["own", 3]]
        })
    };
    let source = r#"<main><Tag v-bind="props" v-on="{ go: save }" label="fixed" /><span data-id="out"><button data-id="own" v-on="{ click: save }">own</button></span></main>"#;
    for backend in ["vdom", "vapor"] {
        let actual = mounted_trace_with_components(
            backend,
            source,
            &[tag],
            json!({"props": {"label": "from-props", "title": "T"}}),
            json!([{"click": "tag"}, {"patch": {"props": {"title": "U"}}}, {"click": "own"}]),
            true,
        );
        assert_eq!(
            actual,
            json!([
                view("T", json!([])),
                view("T", json!(["save"])),
                view("U", json!(["save"])),
                view("U", json!(["save", "save"])),
                {"tree": [], "events": ["save", "save"], "identities": []}
            ]),
            "{backend}"
        );
    }
}
