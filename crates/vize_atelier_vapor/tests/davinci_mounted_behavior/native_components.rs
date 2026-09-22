//! Native S3 components, slot outlets and root fragments through the published
//! runtimes. Child components are compiled by the parent's backend and lane.
//! Every expectation is written out independently of either compiler.

use super::native_control::element;
use super::runtime::{Child, mounted_trace_with_components};
use serde_json::{Value, json};

const BACKENDS: [&str; 3] = ["vdom", "vapor", "vapor-legacy"];

fn assert_components(
    source: &str,
    children: &[Child<'_>],
    context: Value,
    steps: Value,
    identities: bool,
    expected: &Value,
) {
    for backend in BACKENDS {
        let actual = mounted_trace_with_components(
            backend,
            source,
            children,
            context.clone(),
            steps.clone(),
            identities,
        );
        assert_eq!(&actual, expected, "{backend}: {source}");
    }
}

#[test]
fn native_component_props_events_and_default_slot_track_parent_state() {
    let counter: Child<'_> = (
        "Counter",
        r#"<button data-id="inc" @click="$emit('bump', step)">{{ label }}: <slot></slot></button>"#,
        &["label", "step"],
        &["bump"],
    );
    let view = |label: &str, total: &str| {
        json!({
            "tree": [element("main", json!({"data-id": "root"}), json!([
                {"tag": "button", "attributes": {"data-id": "inc"}, "disabled": false, "children": [
                    format!("{label}: "),
                    element("b", json!({"data-id": "slot"}), json!([total]))
                ]},
                element("span", json!({"data-id": "out"}), json!([total]))
            ]))],
            "events": [],
            "identities": [["root", 0], ["inc", 1], ["slot", 2], ["out", 3]]
        })
    };
    assert_components(
        r#"<main data-id="root"><Counter :label="title" :step="2" @bump="total += $event"><b data-id="slot">{{ total }}</b></Counter><span data-id="out">{{ total }}</span></main>"#,
        &[counter],
        json!({"title": "A", "total": 0}),
        json!([{"click": "inc"}, {"patch": {"title": "B"}}, {"click": "inc"}]),
        true,
        &json!([
            view("A", "0"),
            view("A", "2"),
            view("B", "2"),
            view("B", "4"),
            {"tree": [], "events": [], "identities": []}
        ]),
    );
}

#[test]
fn native_slot_outlets_render_fallbacks_and_provided_content() {
    let frame: Child<'_> = (
        "Frame",
        r#"<section><slot name="head"><i>default head</i></slot><slot :n="count"><i>fallback {{ count }}</i></slot></section>"#,
        &["count"],
        &[],
    );
    let view = |n: &str| {
        json!({
            "tree": [element("main", json!({}), json!([element("section", json!({}), json!([
                element("i", json!({}), json!(["default head"])),
                element("b", json!({}), json!([format!("body {n}")]))
            ]))]))],
            "events": []
        })
    };
    assert_components(
        r#"<main><Frame :count="n"><b>body {{ n }}</b></Frame></main>"#,
        &[frame],
        json!({"n": 1}),
        json!([{"patch": {"n": 2}}]),
        false,
        &json!([view("1"), view("2"), {"tree": [], "events": []}]),
    );
    // With no content provided, both fallbacks render and track child props.
    let frame: Child<'_> = (
        "Frame",
        r#"<section><slot name="head"><i>default head</i></slot><slot :n="count"><i>fallback {{ count }}</i></slot></section>"#,
        &["count"],
        &[],
    );
    let view = |n: &str| {
        json!({
            "tree": [element("section", json!({}), json!([
                element("i", json!({}), json!(["default head"])),
                element("i", json!({}), json!([format!("fallback {n}")]))
            ]))],
            "events": []
        })
    };
    assert_components(
        r#"<Frame :count="n" />"#,
        &[frame],
        json!({"n": 1}),
        json!([{"patch": {"n": 3}}]),
        false,
        &json!([view("1"), view("3"), {"tree": [], "events": []}]),
    );
}

#[test]
fn native_components_inside_keyed_loops_follow_their_items() {
    let tag: Child<'_> = ("Tag", r#"<em>{{ label }}</em>"#, &["label"], &[]);
    let view = |labels: &[&str]| {
        let items: std::vec::Vec<_> = labels
            .iter()
            .map(|label| {
                element(
                    "li",
                    json!({}),
                    json!([element("em", json!({}), json!([label]))]),
                )
            })
            .collect();
        json!({"tree": [element("ul", json!({}), Value::Array(items))], "events": []})
    };
    assert_components(
        r#"<ul><li v-for="item in items" :key="item.id"><Tag :label="item.label" /></li></ul>"#,
        &[tag],
        json!({"items": [{"id": "a", "label": "A"}, {"id": "b", "label": "B"}]}),
        json!([
            {"patch": {"items": [{"id": "b", "label": "B2"}, {"id": "a", "label": "A"}]}},
            {"patch": {"items": []}}
        ]),
        false,
        &json!([
            view(&["A", "B"]),
            view(&["B2", "A"]),
            view(&[]),
            {"tree": [], "events": []}
        ]),
    );
}

#[test]
fn native_root_fragments_and_text_update_each_node() {
    // The VDOM trace harness mounts single-root render functions only, so the
    // fragment is checked on both Vapor lanes against the written expectation.
    let fragment = |backend| {
        crate::mounted_trace(
            backend,
            r#"<b>{{ a }}</b><i :title="c">x</i><span>{{ a }}-{{ c }}</span>"#,
            json!({"a": "A", "c": "C"}),
            json!([{"patch": {"a": "A2"}}, {"patch": {"c": 3}}]),
        )
    };
    let view = |a: &str, c: &str| {
        json!({
            "tree": [
                element("b", json!({}), json!([a])),
                element("i", json!({"title": c}), json!(["x"])),
                element("span", json!({}), json!([format!("{a}-{c}")]))
            ],
            "events": []
        })
    };
    for backend in ["vapor", "vapor-legacy"] {
        assert_eq!(
            fragment(backend),
            json!([
                view("A", "C"),
                view("A2", "C"),
                view("A2", "3"),
                {"tree": [], "events": []}
            ]),
            "{backend}"
        );
    }
    let view = |text: &str| json!({"tree": [text], "events": []});
    for backend in ["vapor", "vapor-legacy"] {
        let actual = crate::mounted_trace(
            backend,
            r#"{{ a }} and {{ c }}"#,
            json!({"a": "A", "c": "C"}),
            json!([{"patch": {"a": null}}, {"patch": {"c": 3}}]),
        );
        assert_eq!(
            actual,
            json!([
                view("A and C"),
                view(" and C"),
                view(" and 3"),
                {"tree": [], "events": []}
            ]),
            "{backend}"
        );
    }
}

#[test]
fn native_dynamic_component_switches_between_registered_children() {
    // Nested child roots: the VDOM runner leaves a text-only component root
    // stale after a prop patch, with or without `<component>`.
    let a: Child<'_> = (
        "PanelA",
        r#"<p data-id="a"><b>A {{ label }}</b></p>"#,
        &["label"],
        &[],
    );
    let b: Child<'_> = (
        "PanelB",
        r#"<p data-id="b"><i>B {{ label }}</i></p>"#,
        &["label"],
        &[],
    );
    let view = |tag: &str, id: &str, text: &str, identity: usize| {
        json!({
            "tree": [element("main", json!({"data-id": "root"}), json!([
                element("p", json!({"data-id": id}), json!([element(tag, json!({}), json!([text]))]))
            ]))],
            "events": [],
            "identities": [["root", 0], [id, identity]]
        })
    };
    assert_components(
        r#"<main data-id="root"><component :is="kind" :label="label"></component></main>"#,
        &[a, b],
        json!({"kind": "PanelA", "label": "x"}),
        json!([{"patch": {"label": "y"}}, {"patch": {"kind": "PanelB"}}, {"patch": {"kind": "PanelA"}}]),
        true,
        &json!([
            view("b", "a", "A x", 1),
            view("b", "a", "A y", 1),
            view("i", "b", "B y", 3),
            view("b", "a", "A y", 5),
            {"tree": [], "events": [], "identities": []}
        ]),
    );
}
