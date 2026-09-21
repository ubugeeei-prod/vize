//! Native S3 expressions, handlers and content directives through the published
//! runtimes. Expectations are written out independently of either compiler.

use super::native_control::element;
use crate::{assert_backends, mounted_trace_with_identity};
use serde_json::{Value, json};

#[test]
fn native_compound_props_classes_and_text_update_in_place() {
    let source = r#"<main><div data-id="card" class="card" :class="{ active: count > 1, [tone]: true }" :style="{ color: color }" :title="'n=' + count">{{ count * 2 }} / {{ label.toUpperCase() }}</div></main>"#;
    let card = |class: &str, color: &str, title: &str, text: &str| {
        let attributes = json!({
            "class": class, "data-id": "card", "style": format!("color: {color};"), "title": title
        });
        json!({
            "tree": [element("main", json!({}), json!([element("div", attributes, json!([text]))]))],
            "events": [],
            "identities": [["card", 1]]
        })
    };
    let expected = json!([
        card("card warm", "red", "n=1", "2 / HI"),
        card("card active warm", "red", "n=2", "4 / HI"),
        card("card active cool", "blue", "n=2", "4 / YO"),
        card("card cool", "blue", "n=0", "0 / YO"),
        {"tree": [], "events": [], "identities": []}
    ]);
    for backend in ["vdom", "vapor", "vapor-legacy"] {
        let actual = mounted_trace_with_identity(
            backend,
            source,
            json!({"count": 1, "tone": "warm", "color": "red", "label": "hi"}),
            json!([
                {"patch": {"count": 2}},
                {"patch": {"tone": "cool", "color": "blue", "label": "yo"}},
                {"patch": {"count": 0}}
            ]),
            false,
            true,
        );
        assert_eq!(actual, expected, "{backend}");
    }
}

#[test]
fn native_inline_handlers_update_state_and_receive_the_event() {
    let source = r#"<main><button data-id="inc" @click="count++">+</button><button data-id="add" @click="count = count + step">add</button><button data-id="rec" @click="record(label + ':' + count)">rec</button><button data-id="arrow" @click="() => record('arrow')">arrow</button><button data-id="evt" @click="record($event.type)">evt</button><span data-id="out">{{ count }}</span></main>"#;
    let view = |count: &str, events: Value| {
        let button = |id: &str, label: &str| json!({"tag": "button", "attributes": {"data-id": id}, "children": [label], "disabled": false});
        json!({
            "tree": [element("main", json!({}), json!([
                button("inc", "+"), button("add", "add"), button("rec", "rec"),
                button("arrow", "arrow"), button("evt", "evt"),
                element("span", json!({"data-id": "out"}), json!([count]))
            ]))],
            "events": events,
            "identities": [["inc", 1], ["add", 2], ["rec", 3], ["arrow", 4], ["evt", 5], ["out", 6]]
        })
    };
    let expected = json!([
        view("0", json!([])),
        view("1", json!([])),
        view("6", json!([])),
        view("6", json!(["L:6"])),
        view("6", json!(["L:6", "arrow"])),
        view("6", json!(["L:6", "arrow", "click"])),
        {"tree": [], "events": ["L:6", "arrow", "click"], "identities": []}
    ]);
    for backend in ["vdom", "vapor", "vapor-legacy"] {
        let actual = mounted_trace_with_identity(
            backend,
            source,
            json!({"count": 0, "step": 5, "label": "L"}),
            json!([
                {"click": "inc"}, {"click": "add"}, {"click": "rec"},
                {"click": "arrow"}, {"click": "evt"}
            ]),
            false,
            true,
        );
        assert_eq!(actual, expected, "{backend}");
    }
}

#[test]
fn native_show_text_and_html_directives_track_state() {
    let actual = assert_backends(
        r#"<main><div v-show="visible && ready">shown</div><span v-text="message + '!'"></span><i v-html="markup"></i></main>"#,
        json!({"visible": true, "ready": false, "message": "hi", "markup": "<b>bold</b>"}),
        json!([
            {"patch": {"ready": true}},
            {"patch": {"visible": false, "message": "yo", "markup": "plain"}}
        ]),
    );
    let view = |style: Option<&str>, text: &str, html: Value| {
        let attributes = style.map_or_else(|| json!({}), |style| json!({"style": style}));
        json!({
            "tree": [element("main", json!({}), json!([
                element("div", attributes, json!(["shown"])),
                element("span", json!({}), json!([text])),
                element("i", json!({}), html)
            ]))],
            "events": []
        })
    };
    let bold = json!([element("b", json!({}), json!(["bold"]))]);
    assert_eq!(
        actual,
        json!([
            view(Some("display: none;"), "hi!", bold.clone()),
            view(None, "hi!", bold),
            view(Some("display: none;"), "yo!", json!(["plain"])),
            {"tree": [], "events": []}
        ])
    );
}
