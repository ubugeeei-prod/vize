//! TS-33: S3 Vapor and official compiler-vapor output must have the same
//! mounted behavior under the same published runtime, including node identity.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types
)]

use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
};

use serde::Deserialize;
use serde_json::{Value, json};
use vize_atelier_core::walk_probe::WalkCounts;
use vize_atelier_vapor::{VaporCompilerOptions, compile_vapor};
use vize_carton::Allocator;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Fixture {
    source: String,
    context: Value,
    steps: Vec<Value>,
    expected: Vec<Value>,
}

fn trace(runner: &str, input: Value) -> Vec<Value> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/tooling/support")
        .join(runner);
    let mut child = Command::new("node")
        .arg(path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("start TS-33 mounted runner (install workspace JS dependencies first)");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.to_string().as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{runner} failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|error| panic!("{error}: {}", String::from_utf8_lossy(&output.stdout)))
}

#[test]
fn s3_branch_matches_official_vapor_state_and_identity_trace() {
    let fixture: Fixture = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/davinci-ts33-vapor-branch.json"
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
        assert!(
            compiled.code.contains(effect),
            "{source}: {}",
            compiled.code
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
