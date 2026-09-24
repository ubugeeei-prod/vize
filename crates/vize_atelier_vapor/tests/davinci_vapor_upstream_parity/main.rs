//! TS-33: S3 Vapor and official compiler-vapor output must have the same
//! mounted behavior under the same published runtime, including node identity.

#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::expect_used,
    clippy::panic,
    clippy::unwrap_used,
    reason = "tests assert by panicking and compare std-string fixtures"
)]

mod trace;

use serde::Deserialize;
use serde_json::{Value, json};
use vize_atelier_core::walk_probe::WalkCounts;
use vize_atelier_vapor::{VaporCompilerOptions, compile_vapor};
use vize_carton::Allocator;

use trace::trace;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Fixture {
    source: String,
    context: Value,
    steps: Vec<Value>,
    expected: Vec<Value>,
}

#[test]
fn s3_branch_matches_official_vapor_state_and_identity_trace() {
    let fixture: Fixture = serde_json::from_str(include_str!(
        "../../../../tests/_fixtures/davinci-ts33-vapor-branch.json"
    ))
    .unwrap();
    assert!(!fixture.steps.is_empty());
    assert_eq!(fixture.expected.len(), fixture.steps.len() + 2);

    let allocator = Allocator::new();
    let before = WalkCounts::snapshot();
    let compiled = compile_vapor(
        &allocator,
        &fixture.source,
        VaporCompilerOptions {
            prefix_identifiers: true,
            ..Default::default()
        },
    );
    assert!(
        compiled.error_messages.is_empty(),
        "{:?}",
        compiled.error_messages
    );
    assert_eq!(
        WalkCounts::snapshot().since(before).total_walks(),
        0,
        "TS-33 must exercise native S3"
    );

    let vize = trace(
        "davinci-mounted-trace.mjs",
        json!({
            "backend": "vapor",
            "code": compiled.code,
            "context": fixture.context,
            "steps": fixture.steps,
            "identities": true,
        }),
    );
    let upstream = trace(
        "davinci-upstream-vapor-trace.mjs",
        json!({
            "source": fixture.source,
            "context": fixture.context,
            "steps": fixture.steps,
        }),
    );
    assert_eq!(vize, fixture.expected, "Vize native S3 trace");
    assert_eq!(upstream, fixture.expected, "official compiler-vapor trace");
    assert_eq!(vize, upstream, "TS-33 behavior-level parity");
}

#[test]
fn s3_style_merge_matches_official_vapor_in_both_authored_orders() {
    let context = json!({
        "theme": {"color": "blue", "backgroundColor": "yellow"},
        "label": "A",
    });
    let steps = json!([
        {"patch": {"theme": {"backgroundColor": "orange"}, "label": "B"}},
        {"patch": {"theme": {}, "label": "C"}},
    ]);
    let view = |style: &str, label: &str| {
        json!({
            "tree": [{
                "tag": "main",
                "attributes": {"data-id": "root"},
                "children": [
                    {"tag": "div", "attributes": {"data-id": "box", "style": style}, "children": [label]},
                    {"tag": "i", "attributes": {"data-id": "tail"}, "children": ["tail"]},
                ],
            }],
            "events": [],
            "identities": [["root", 0], ["box", 1], ["tail", 2]],
        })
    };
    for (source, first_style, second_style, effect) in [
        (
            r#"<main data-id="root"><div data-id="box" style="color: red;" :style="theme">{{ label }}</div><i data-id="tail">tail</i></main>"#,
            "color: blue; background-color: yellow;",
            "color: red; background-color: orange;",
            "_setStyle(n1, [\"color: red;\", _ctx.theme])",
        ),
        (
            r#"<main data-id="root"><div data-id="box" :style="theme" style="color: red;">{{ label }}</div><i data-id="tail">tail</i></main>"#,
            "background-color: yellow; color: red;",
            "background-color: orange; color: red;",
            "_setStyle(n1, [_ctx.theme, \"color: red;\"])",
        ),
    ] {
        let expected = vec![
            view(first_style, "A"),
            view(second_style, "B"),
            view("color: red;", "C"),
            json!({"tree": [], "events": [], "identities": []}),
        ];
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
        let expected_code = format!(
            r#"import {{ child as _child, txt as _txt, toDisplayString as _toDisplayString, setText as _setText, setStyle as _setStyle, renderEffect as _renderEffect, template as _template }} from 'vue';
const t0 = _template("<main data-id=\"root\"><div data-id=\"box\"> </div><i data-id=\"tail\">tail</i></main>", true)

export function render(_ctx) {{
  const n0 = t0()
  const n1 = _child(n0)
  const x1 = _txt(n1)
  _renderEffect(() => {{
    {effect}
    _setText(x1, _toDisplayString(_ctx.label))
  }})
  return n0
}}
"#
        );
        assert_eq!(
            compiled.code, expected_code,
            "{source}: generated Vapor code"
        );
        assert_eq!(
            WalkCounts::snapshot().since(before).total_walks(),
            0,
            "{source}: TS-33 must exercise native S3"
        );
        let vize = trace(
            "davinci-mounted-trace.mjs",
            json!({
                "backend": "vapor",
                "code": compiled.code,
                "context": context.clone(),
                "steps": steps.clone(),
                "identities": true,
            }),
        );
        let upstream = trace(
            "davinci-upstream-vapor-trace.mjs",
            json!({"source": source, "context": context.clone(), "steps": steps.clone()}),
        );
        assert_eq!(vize, expected, "{source}: Vize native S3 trace");
        assert_eq!(
            upstream, expected,
            "{source}: official compiler-vapor trace"
        );
        assert_eq!(vize, upstream, "{source}: TS-33 behavior-level parity");
    }
}

#[test]
fn s3_component_models_match_official_vapor_updates() {
    for (model, prop) in [("v-model", "modelValue"), ("v-model:title.trim", "title")] {
        let source = format!(
            r#"<main data-id="root"><MyComp {model}="value" /><span data-id="mirror">{{{{ value }}}}</span></main>"#
        );
        let event = format!("update:{prop}");
        let child = format!(
            r#"<button data-id="child" @click="send('{event}', {prop} + '!')">{{{{ {prop} }}}}</button>"#
        );
        let allocator = Allocator::new();
        let before = WalkCounts::snapshot();
        let compiled = compile_vapor(
            &allocator,
            &source,
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
            "{source}: parent must exercise native S3"
        );
        let child_code = compile_vapor(
            &allocator,
            &child,
            VaporCompilerOptions {
                prefix_identifiers: true,
                ..Default::default()
            },
        );
        assert!(
            child_code.error_messages.is_empty(),
            "{child}: {:?}",
            child_code.error_messages
        );
        let context = json!({"value": "A"});
        let steps = json!([{"click": "child"}, {"patch": {"value": "P"}}]);
        let vize = trace(
            "davinci-mounted-trace.mjs",
            json!({
                "backend": "vapor",
                "code": compiled.code,
                "context": context,
                "steps": steps,
                "identities": true,
                "components": {"MyComp": {"code": child_code.code, "props": [prop], "emits": [event]}},
            }),
        );
        let upstream = trace(
            "davinci-upstream-vapor-trace.mjs",
            json!({
                "source": source,
                "context": context,
                "steps": steps,
                "components": {"MyComp": {"source": child, "props": [prop], "emits": [event]}},
            }),
        );
        assert_eq!(vize, upstream, "{model}: mounted component model parity");
        assert_eq!(vize.len(), 4);
        for (snapshot, expected) in vize.iter().take(3).zip(["A", "A!", "P"]) {
            let children = snapshot["tree"][0]["children"].as_array().unwrap();
            assert_eq!(children[0]["children"][0], expected, "{model}: child prop");
            assert_eq!(
                children[1]["children"][0], expected,
                "{model}: parent assignment"
            );
        }
        assert_eq!(vize[3]["tree"], json!([]), "{model}: unmount");
    }
}

fn assert_native_upstream_trace(source: &str, context: Value, steps: Value, expected: Vec<Value>) {
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
        "{source}: TS-33 must exercise native S3"
    );
    let vize = trace(
        "davinci-mounted-trace.mjs",
        json!({
            "backend": "vapor",
            "code": compiled.code,
            "context": context.clone(),
            "steps": steps.clone(),
            "identities": true,
        }),
    );
    let upstream = trace(
        "davinci-upstream-vapor-trace.mjs",
        json!({"source": source, "context": context, "steps": steps}),
    );
    assert_eq!(vize, expected, "{source}: Vize native S3 trace");
    assert_eq!(
        upstream, expected,
        "{source}: official compiler-vapor trace"
    );
    assert_eq!(vize, upstream, "{source}: TS-33 behavior-level parity");
}

#[test]
fn s3_cloak_matches_official_vapor_across_branch_recreation() {
    let source = r#"<main data-id="root"><div v-if="open" v-cloak data-id="cloak" :title="tip">{{ label }}</div><p data-id="tail">tail</p></main>"#;
    let cloak = |title: &str, label: &str| json!({"tag": "div", "attributes": {"data-id": "cloak", "title": title}, "children": [label]});
    let tail = json!({"tag": "p", "attributes": {"data-id": "tail"}, "children": ["tail"]});
    let view = |children: Value, identities: Value| {
        json!({
            "tree": [{"tag": "main", "attributes": {"data-id": "root"}, "children": children}],
            "events": [],
            "identities": identities,
        })
    };
    assert_native_upstream_trace(
        source,
        json!({"open": true, "tip": "first", "label": "A"}),
        json!([
            {"patch": {"tip": "second", "label": "B"}},
            {"patch": {"open": false}},
            {"patch": {"open": true}},
        ]),
        vec![
            view(
                json!([cloak("first", "A"), tail]),
                json!([["root", 0], ["cloak", 1], ["tail", 2]]),
            ),
            view(
                json!([cloak("second", "B"), tail]),
                json!([["root", 0], ["cloak", 1], ["tail", 2]]),
            ),
            view(json!([tail]), json!([["root", 0], ["tail", 2]])),
            view(
                json!([cloak("second", "B"), tail]),
                json!([["root", 0], ["cloak", 3], ["tail", 2]]),
            ),
            json!({"tree": [], "events": [], "identities": []}),
        ],
    );
}

#[test]
fn s3_textarea_model_matches_official_vapor_input_and_external_patch() {
    let source = r#"<section data-id="root"><textarea data-id="field" v-model="content"></textarea><p data-id="label">{{ content }}</p></section>"#;
    let view = |content: &str| {
        json!({
            "tree": [{
                "tag": "section",
                "attributes": {"data-id": "root"},
                "children": [
                    {"tag": "textarea", "attributes": {"data-id": "field"}, "children": [], "value": content},
                    {"tag": "p", "attributes": {"data-id": "label"}, "children": [content]},
                ],
            }],
            "events": [],
            "identities": [["root", 0], ["field", 1], ["label", 2]],
        })
    };
    assert_native_upstream_trace(
        source,
        json!({"content": "initial"}),
        json!([
            {"event": "input", "selector": "textarea", "value": "typed"},
            {"patch": {"content": "external"}},
        ]),
        vec![
            view("initial"),
            view("typed"),
            view("external"),
            json!({"tree": [], "events": [], "identities": []}),
        ],
    );
}

#[test]
fn s3_phrasing_elements_match_official_vapor_updates_in_place() {
    let source = r#"<p data-id="root"><code data-id="code" :title="tip">{{ label }}</code><mark data-id="mark">new</mark><time data-id="time" :datetime="date">{{ date }}</time></p>"#;
    let view = |tip: &str, label: &str, date: &str| {
        json!({
            "tree": [{
                "tag": "p",
                "attributes": {"data-id": "root"},
                "children": [
                    {"tag": "code", "attributes": {"data-id": "code", "title": tip}, "children": [label]},
                    {"tag": "mark", "attributes": {"data-id": "mark"}, "children": ["new"]},
                    {"tag": "time", "attributes": {"data-id": "time", "datetime": date}, "children": [date]},
                ],
            }],
            "events": [],
            "identities": [["root", 0], ["code", 1], ["mark", 2], ["time", 3]],
        })
    };
    assert_native_upstream_trace(
        source,
        json!({"tip": "first", "label": "A", "date": "2026-09-23"}),
        json!([{"patch": {"tip": "second", "label": "B", "date": "2026-09-24"}}]),
        vec![
            view("first", "A", "2026-09-23"),
            view("second", "B", "2026-09-24"),
            json!({"tree": [], "events": [], "identities": []}),
        ],
    );
}
