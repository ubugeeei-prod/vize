//! Native `<template v-if>` / `<template v-for>` fragments, observed through the
//! published runtimes. Expectations are written out independently of either
//! compiler: DOM trees and element identities per step.

use crate::mounted_trace_with_identity;
use serde_json::{Value, json};

use super::native_control::element;

pub(super) const BRANCH: &str = r#"<main data-id="root"><template v-if="open"><b data-id="a">{{ a }}</b><i data-id="b">{{ b }}</i></template><span v-else data-id="c">closed</span><em data-id="tail">t</em></main>"#;
pub(super) const LOOP: &str = r#"<main data-id="root"><ul data-id="list"><template v-for="row in rows" :key="row.id"><li :data-id="'l' + row.id">{{ row.label }}</li><li :data-id="'n' + row.id">{{ row.note }}</li></template></ul></main>"#;
pub(super) const TEXT: &str = r#"<main data-id="root"><template v-if="on">on {{ n }}</template><template v-else>off</template></main>"#;

fn observation(children: Value, identities: Value) -> Value {
    json!({
        "tree": [element("main", json!({"data-id": "root"}), children)],
        "events": [],
        "identities": identities
    })
}

fn unmounted() -> Value {
    json!({"tree": [], "events": [], "identities": []})
}

fn assert_identity_trace(source: &str, context: Value, steps: Value, expected: &Value) {
    for backend in ["vdom", "vapor", "vapor-legacy"] {
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
fn native_template_branch_swaps_its_fragment_as_a_unit() {
    let a = |text| element("b", json!({"data-id": "a"}), json!([text]));
    let b = |text| element("i", json!({"data-id": "b"}), json!([text]));
    let closed = element("span", json!({"data-id": "c"}), json!(["closed"]));
    let tail = element("em", json!({"data-id": "tail"}), json!(["t"]));
    let expected = json!([
        observation(
            json!([a("A"), b("B"), tail]),
            json!([["root", 0], ["a", 1], ["b", 2], ["tail", 3]])
        ),
        // Text updates patch the live fragment in place.
        observation(
            json!([a("A2"), b("B"), tail]),
            json!([["root", 0], ["a", 1], ["b", 2], ["tail", 3]])
        ),
        observation(
            json!([closed, tail]),
            json!([["root", 0], ["c", 4], ["tail", 3]])
        ),
        // Reopening mounts a fresh fragment before the untouched tail.
        observation(
            json!([a("A2"), b("B2"), tail]),
            json!([["root", 0], ["a", 5], ["b", 6], ["tail", 3]])
        ),
        unmounted()
    ]);
    assert_identity_trace(
        BRANCH,
        json!({"open": true, "a": "A", "b": "B"}),
        json!([
            {"patch": {"a": "A2"}},
            {"patch": {"open": false}},
            {"patch": {"open": true, "b": "B2"}}
        ]),
        &expected,
    );
}

#[test]
fn native_template_loop_moves_each_item_fragment_by_key() {
    let item = |id: &str, label: &str, note: &str| {
        [
            element(
                "li",
                json!({"data-id": vize_carton::cstr!("l{id}").as_str()}),
                json!([label]),
            ),
            element(
                "li",
                json!({"data-id": vize_carton::cstr!("n{id}").as_str()}),
                json!([note]),
            ),
        ]
    };
    let list = |items: &[[Value; 2]]| {
        let children: Vec<Value> = items.iter().flat_map(|pair| pair.iter().cloned()).collect();
        json!([element("ul", json!({"data-id": "list"}), json!(children))])
    };
    let row = |id: u32, label: &str, note: &str| json!({"id": id, "label": label, "note": note});
    let expected = json!([
        observation(
            list(&[item("1", "A", "a"), item("2", "B", "b")]),
            json!([
                ["root", 0],
                ["list", 1],
                ["l1", 2],
                ["n1", 3],
                ["l2", 4],
                ["n2", 5]
            ])
        ),
        // Reordering moves both nodes of each item and keeps their identities.
        observation(
            list(&[item("2", "B", "b"), item("1", "A", "a")]),
            json!([
                ["root", 0],
                ["list", 1],
                ["l2", 4],
                ["n2", 5],
                ["l1", 2],
                ["n1", 3]
            ])
        ),
        observation(
            list(&[
                item("3", "C", "c"),
                item("2", "B", "b"),
                item("1", "A", "a")
            ]),
            json!([
                ["root", 0],
                ["list", 1],
                ["l3", 6],
                ["n3", 7],
                ["l2", 4],
                ["n2", 5],
                ["l1", 2],
                ["n1", 3]
            ])
        ),
        observation(
            list(&[item("3", "C", "c"), item("1", "A2", "a")]),
            json!([
                ["root", 0],
                ["list", 1],
                ["l3", 6],
                ["n3", 7],
                ["l1", 2],
                ["n1", 3]
            ])
        ),
        unmounted()
    ]);
    assert_identity_trace(
        LOOP,
        json!({"rows": [row(1, "A", "a"), row(2, "B", "b")]}),
        json!([
            {"patch": {"rows": [row(2, "B", "b"), row(1, "A", "a")]}},
            {"patch": {"rows": [row(3, "C", "c"), row(2, "B", "b"), row(1, "A", "a")]}},
            {"patch": {"rows": [row(3, "C", "c"), row(1, "A2", "a")]}}
        ]),
        &expected,
    );
}

#[test]
fn native_template_text_branches_render_bare_text() {
    let expected = json!([
        observation(json!(["on 1"]), json!([["root", 0]])),
        observation(json!(["on 2"]), json!([["root", 0]])),
        observation(json!(["off"]), json!([["root", 0]])),
        observation(json!(["on 2"]), json!([["root", 0]])),
        unmounted()
    ]);
    assert_identity_trace(
        TEXT,
        json!({"on": true, "n": 1}),
        json!([
            {"patch": {"n": 2}},
            {"patch": {"on": false}},
            {"patch": {"on": true}}
        ]),
        &expected,
    );
}
