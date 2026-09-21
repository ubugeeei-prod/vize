//! Native S3 branches and loops, observed through the published runtimes.
//! Every expectation is written out independently of either compiler: DOM
//! trees, delivered events, and element identities per step.

use crate::mounted_trace_with_identity;
use serde_json::{Value, json};

pub(super) fn element(tag: &str, attributes: Value, children: Value) -> Value {
    json!({"tag": tag, "attributes": attributes, "children": children})
}

fn button(attributes: Value, label: &str) -> Value {
    json!({"tag": "button", "attributes": attributes, "children": [label], "disabled": false})
}

fn observation(children: Value, events: usize, identities: Value) -> Value {
    json!({
        "tree": [element("main", json!({"data-id": "root"}), children)],
        "events": vec!["save"; events],
        "identities": identities
    })
}

fn unmounted(events: usize) -> Value {
    json!({"tree": [], "events": vec!["save"; events], "identities": []})
}

fn assert_identity_trace(
    backends: &[&str],
    source: &str,
    context: Value,
    steps: Value,
    expected: &Value,
) {
    for backend in backends {
        let actual = mounted_trace_with_identity(
            backend,
            source,
            context.clone(),
            steps.clone(),
            false,
            true,
        );
        assert_eq!(&actual, expected, "{backend}: {source}");
    }
}

#[test]
fn native_branch_chain_swaps_nodes_and_keeps_live_branches() {
    let source = r#"<main data-id="root"><button v-if="mode" data-id="yes" :title="label" @click="save">{{ label }}</button><span v-else-if="alt" data-id="alt">{{ label }}</span><i v-else data-id="none">none</i><b data-id="tail">{{ tail }}</b></main>"#;
    let yes = |label| button(json!({"data-id": "yes", "title": label}), label);
    let alt = |label| element("span", json!({"data-id": "alt"}), json!([label]));
    let none = element("i", json!({"data-id": "none"}), json!(["none"]));
    let tail = |text| element("b", json!({"data-id": "tail"}), json!([text]));
    let ids = |branch: &str, id: usize| json!([["root", 0], [branch, id], ["tail", 2]]);
    let expected = json!([
        observation(json!([yes("A"), tail("T")]), 0, ids("yes", 1)),
        observation(json!([yes("A"), tail("T")]), 1, ids("yes", 1)),
        observation(json!([yes("B"), tail("T")]), 1, ids("yes", 1)),
        observation(json!([none, tail("T")]), 1, ids("none", 3)),
        observation(json!([alt("B"), tail("T")]), 1, ids("alt", 4)),
        observation(json!([alt("C"), tail("U")]), 1, ids("alt", 4)),
        observation(json!([yes("C"), tail("U")]), 1, ids("yes", 5)),
        observation(json!([yes("C"), tail("U")]), 2, ids("yes", 5)),
        unmounted(2)
    ]);
    assert_identity_trace(
        &["vdom", "vapor"],
        source,
        json!({"mode": true, "alt": false, "label": "A", "tail": "T"}),
        json!([
            {"click": "yes"},
            {"patch": {"label": "B"}},
            {"patch": {"mode": false}},
            {"patch": {"alt": true}},
            {"patch": {"label": "C", "tail": "U"}},
            {"patch": {"mode": true}},
            {"click": "yes"}
        ]),
        &expected,
    );
}

const LOOP_STEPS: &str = r#"[
    {"click": "a"},
    {"patch": {"items": [{"id": "b", "label": "B2"}, {"id": "a", "label": "A2"}]}},
    {"click": "a"},
    {"patch": {"items": [{"id": "c", "label": "C"}, {"id": "b", "label": "B2"}, {"id": "a", "label": "A2"}]}},
    {"patch": {"items": [{"id": "c", "label": "C2"}, {"id": "a", "label": "A3"}]}},
    {"click": "c"},
    {"patch": {"items": [], "item": "outer changed"}},
    {"patch": {"items": [{"id": "a", "label": "A5"}]}},
    {"click": "a"}
]"#;

/// One button's `(data-id, title, label)`.
type Row<'a> = (&'a str, &'a str, &'a str);
/// The buttons rendered after one step and the identity of each.
type Step<'a> = (&'a [Row<'a>], &'a [usize]);

fn loop_trace(rows: [Step<'_>; 10]) -> Value {
    let mut trace = std::vec::Vec::new();
    for (step, (buttons, identities)) in rows.into_iter().enumerate() {
        let events = [0, 1, 1, 2, 2, 2, 3, 3, 3, 4][step];
        let tail = if step < 7 { "outer" } else { "outer changed" };
        let mut children: std::vec::Vec<_> = buttons
            .iter()
            .map(|(id, title, label)| button(json!({"data-id": id, "title": title}), label))
            .collect();
        children.push(element("span", json!({"data-id": "tail"}), json!([tail])));
        let mut ids = vec![json!(["root", 0])];
        ids.extend(
            buttons
                .iter()
                .zip(identities)
                .map(|((id, ..), n)| json!([id, n])),
        );
        ids.push(json!(["tail", 3]));
        trace.push(observation(
            Value::Array(children),
            events,
            Value::Array(ids),
        ));
    }
    trace.push(unmounted(4));
    Value::Array(trace)
}

#[test]
fn native_keyed_loop_moves_retained_nodes_and_scopes_aliases() {
    let source = r#"<main data-id="root"><button v-for="(item, position) in items" :key="item.id" :data-id="item.id" :title="position" @click="save">{{ item.label }}</button><span data-id="tail">{{ item }}</span></main>"#;
    let context =
        json!({"items": [{"id": "a", "label": "A"}, {"id": "b", "label": "B"}], "item": "outer"});
    let expected = loop_trace([
        (&[("a", "0", "A"), ("b", "1", "B")], &[1, 2]),
        (&[("a", "0", "A"), ("b", "1", "B")], &[1, 2]),
        (&[("b", "0", "B2"), ("a", "1", "A2")], &[2, 1]),
        (&[("b", "0", "B2"), ("a", "1", "A2")], &[2, 1]),
        (
            &[("c", "0", "C"), ("b", "1", "B2"), ("a", "2", "A2")],
            &[4, 2, 1],
        ),
        (&[("c", "0", "C2"), ("a", "1", "A3")], &[4, 1]),
        (&[("c", "0", "C2"), ("a", "1", "A3")], &[4, 1]),
        (&[], &[]),
        (&[("a", "0", "A5")], &[5]),
        (&[("a", "0", "A5")], &[5]),
    ]);
    let steps: Value = serde_json::from_str(LOOP_STEPS).unwrap();
    assert_identity_trace(&["vdom", "vapor"], source, context, steps, &expected);
}

#[test]
fn native_unkeyed_loop_patches_nodes_in_place_by_position() {
    let source = r#"<main data-id="root"><button v-for="(item, position) in items" :data-id="item.id" :title="position" @click="save">{{ item.label }}</button><span data-id="tail">{{ item }}</span></main>"#;
    let context =
        json!({"items": [{"id": "a", "label": "A"}, {"id": "b", "label": "B"}], "item": "outer"});
    let expected = loop_trace([
        (&[("a", "0", "A"), ("b", "1", "B")], &[1, 2]),
        (&[("a", "0", "A"), ("b", "1", "B")], &[1, 2]),
        (&[("b", "0", "B2"), ("a", "1", "A2")], &[1, 2]),
        (&[("b", "0", "B2"), ("a", "1", "A2")], &[1, 2]),
        (
            &[("c", "0", "C"), ("b", "1", "B2"), ("a", "2", "A2")],
            &[1, 2, 4],
        ),
        (&[("c", "0", "C2"), ("a", "1", "A3")], &[1, 2]),
        (&[("c", "0", "C2"), ("a", "1", "A3")], &[1, 2]),
        (&[], &[]),
        (&[("a", "0", "A5")], &[5]),
        (&[("a", "0", "A5")], &[5]),
    ]);
    let steps: Value = serde_json::from_str(LOOP_STEPS).unwrap();
    assert_identity_trace(&["vdom", "vapor"], source, context, steps, &expected);
}

#[test]
fn nested_loops_inside_branches_clear_only_their_own_nodes() {
    let source = r#"<main data-id="root"><section v-if="open" data-id="panel"><b data-id="title">{{ title }}</b><span v-for="row in rows" :key="row.id" :data-id="row.id">{{ row.label }}</span><ul data-id="cells"><li v-for="cell in cells" :key="cell" :data-id="cell">{{ cell }}</li></ul></section><i data-id="tail">tail</i></main>"#;
    let tail = element("i", json!({"data-id": "tail"}), json!(["tail"]));
    let panel = |title: &str, rows: &[(&str, &str)], cells: &[&str]| {
        let mut children = vec![element("b", json!({"data-id": "title"}), json!([title]))];
        for (id, label) in rows {
            children.push(element("span", json!({"data-id": id}), json!([label])));
        }
        let cells: std::vec::Vec<_> = cells
            .iter()
            .map(|cell| element("li", json!({"data-id": cell}), json!([cell])))
            .collect();
        children.push(element(
            "ul",
            json!({"data-id": "cells"}),
            Value::Array(cells),
        ));
        element(
            "section",
            json!({"data-id": "panel"}),
            Value::Array(children),
        )
    };
    let expected = json!([
        observation(
            json!([
                panel("T", &[("r1", "R1"), ("r2", "R2")], &["c1", "c2"]),
                tail
            ]),
            0,
            json!([
                ["root", 0],
                ["panel", 1],
                ["title", 2],
                ["r1", 3],
                ["r2", 4],
                ["cells", 5],
                ["c1", 6],
                ["c2", 7],
                ["tail", 8]
            ])
        ),
        // Emptying a loop beside its title must keep the title node.
        observation(
            json!([panel("T", &[], &["c1", "c2"]), tail]),
            0,
            json!([
                ["root", 0],
                ["panel", 1],
                ["title", 2],
                ["cells", 5],
                ["c1", 6],
                ["c2", 7],
                ["tail", 8]
            ])
        ),
        observation(
            json!([panel("T", &[], &[]), tail]),
            0,
            json!([
                ["root", 0],
                ["panel", 1],
                ["title", 2],
                ["cells", 5],
                ["tail", 8]
            ])
        ),
        observation(
            json!([panel("T", &[("r3", "R3")], &["c3"]), tail]),
            0,
            json!([
                ["root", 0],
                ["panel", 1],
                ["title", 2],
                ["r3", 9],
                ["cells", 5],
                ["c3", 10],
                ["tail", 8]
            ])
        ),
        observation(json!([tail]), 0, json!([["root", 0], ["tail", 8]])),
        observation(
            json!([panel("T2", &[("r3", "R3")], &["c3"]), tail]),
            0,
            json!([
                ["root", 0],
                ["panel", 11],
                ["title", 12],
                ["r3", 13],
                ["cells", 14],
                ["c3", 15],
                ["tail", 8]
            ])
        ),
        unmounted(0)
    ]);
    // The retained lane shares the generator's fast-removal decision; it once
    // cleared the title together with the emptied rows.
    assert_identity_trace(
        &["vdom", "vapor", "vapor-legacy"],
        source,
        json!({"open": true, "title": "T", "rows": [{"id": "r1", "label": "R1"}, {"id": "r2", "label": "R2"}], "cells": ["c1", "c2"]}),
        json!([
            {"patch": {"rows": []}},
            {"patch": {"cells": []}},
            {"patch": {"rows": [{"id": "r3", "label": "R3"}], "cells": ["c3"]}},
            {"patch": {"open": false}},
            {"patch": {"open": true, "title": "T2"}}
        ]),
        &expected,
    );
}
