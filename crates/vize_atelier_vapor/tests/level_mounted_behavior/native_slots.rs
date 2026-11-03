//! Native named and scoped slots through the published runtimes. Child
//! components are compiled by the parent's backend and lane; expectations are
//! written out independently of either compiler.

use super::native_control::element;
use super::runtime::{Child, mounted_trace_with_components};
use serde_json::{Value, json};

pub(super) const LIST: &str = r#"<main data-id="root"><List :rows="rows"><template #row="{ row }"><b :data-id="'b' + row.id">{{ row.label }}</b></template><template #foot>total {{ rows.length }}</template></List></main>"#;
pub(super) const COUNTER: &str = r#"<main data-id="root"><Counter :count="n" v-slot="p"><b data-id="v">{{ p.n }}/{{ p.double }}</b></Counter></main>"#;

fn assert_slots(source: &str, child: Child<'_>, context: Value, steps: Value, expected: &Value) {
    for backend in ["vdom", "vapor", "vapor-legacy"] {
        let actual = mounted_trace_with_components(
            backend,
            source,
            &[child],
            context.clone(),
            steps.clone(),
            true,
        );
        assert_eq!(&actual, expected, "{backend}: {source}");
    }
}

fn observation(children: Value, identities: Value) -> Value {
    json!({
        "tree": [element("main", json!({"data-id": "root"}), children)],
        "events": [],
        "identities": identities
    })
}

#[test]
fn native_named_and_scoped_slots_render_through_the_child_outlets() {
    let list: Child<'_> = (
        "List",
        r#"<section data-id="list"><div v-for="row in rows" :key="row.id" :data-id="'r' + row.id"><slot name="row" :row="row"></slot></div><p data-id="foot"><slot name="foot"></slot></p></section>"#,
        &["rows"],
        &[],
    );
    let row = |id: &str, label: &str| {
        element(
            "div",
            json!({"data-id": vize_carton::cstr!("r{id}").as_str()}),
            json!([element(
                "b",
                json!({"data-id": vize_carton::cstr!("b{id}").as_str()}),
                json!([label])
            )]),
        )
    };
    let section = |rows: Vec<Value>, total: &str| {
        let mut children = rows;
        children.push(element("p", json!({"data-id": "foot"}), json!([total])));
        json!([element(
            "section",
            json!({"data-id": "list"}),
            json!(children)
        )])
    };
    let expected = json!([
        observation(
            section(vec![row("1", "A"), row("2", "B")], "total 2"),
            json!([["root", 0], ["list", 1], ["r1", 2], ["b1", 3], ["r2", 4], ["b2", 5], ["foot", 6]])
        ),
        // Reordering moves each row with its slot content.
        observation(
            section(vec![row("2", "B"), row("1", "A")], "total 2"),
            json!([["root", 0], ["list", 1], ["r2", 4], ["b2", 5], ["r1", 2], ["b1", 3], ["foot", 6]])
        ),
        observation(
            section(vec![row("2", "B2"), row("1", "A"), row("3", "C")], "total 3"),
            json!([
                ["root", 0],
                ["list", 1],
                ["r2", 4],
                ["b2", 5],
                ["r1", 2],
                ["b1", 3],
                ["r3", 7],
                ["b3", 8],
                ["foot", 6]
            ])
        ),
        {"tree": [], "events": [], "identities": []}
    ]);
    let rows = |items: &[(u32, &str)]| {
        json!(
            items
                .iter()
                .map(|(id, label)| json!({"id": id, "label": label}))
                .collect::<Vec<_>>()
        )
    };
    assert_slots(
        LIST,
        list,
        json!({"rows": rows(&[(1, "A"), (2, "B")])}),
        json!([
            {"patch": {"rows": rows(&[(2, "B"), (1, "A")])}},
            {"patch": {"rows": rows(&[(2, "B2"), (1, "A"), (3, "C")])}}
        ]),
        &expected,
    );
}

#[test]
fn native_plain_identifier_slot_params_read_the_props_object() {
    let counter: Child<'_> = (
        "Counter",
        r#"<p data-id="c"><slot :n="count" :double="count * 2"></slot></p>"#,
        &["count"],
        &[],
    );
    let view = |text: &str| {
        observation(
            json!([element(
                "p",
                json!({"data-id": "c"}),
                json!([element("b", json!({"data-id": "v"}), json!([text]))])
            )]),
            json!([["root", 0], ["c", 1], ["v", 2]]),
        )
    };
    let expected = json!([
        view("1/2"),
        view("2/4"),
        view("5/10"),
        {"tree": [], "events": [], "identities": []}
    ]);
    assert_slots(
        COUNTER,
        counter,
        json!({"n": 1}),
        json!([{"patch": {"n": 2}}, {"patch": {"n": 5}}]),
        &expected,
    );
}
