//! Computed Teleport prop keys remove stale runtime props across all Vapor lanes.
//!
//! This is its own integration binary because the walk and expression-parse
//! probes are process-global. The retained control must not race native-only
//! probe assertions in the broader upstream parity binary.

#![expect(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::expect_used,
    clippy::panic,
    clippy::unwrap_used,
    reason = "tests assert by panicking and compare std-string fixtures"
)]

mod trace;

use serde_json::{Value, json};
use vize_atelier_core::{expr_parse_probe, walk_probe::WalkCounts};
use vize_atelier_vapor::{VaporCompilerOptions, compile_vapor};
use vize_carton::Allocator;

const SOURCE: &str = r##"<main data-id="root"><div id="target-a" data-id="a"></div><div id="target-b" data-id="b"></div><Teleport to="#target-a" :[flag]="enabled"><button data-id="button" @click="record(label)">{{ label }}</button></Teleport><i data-id="tail">tail</i></main>"##;

fn view(destination: &str, label: &str, events: Value) -> Value {
    let button = json!({
        "tag": "button",
        "attributes": {"data-id": "button"},
        "children": [label],
        "disabled": false,
    });
    let mut children = vec![
        json!({
            "tag": "div",
            "attributes": {"data-id": "a", "id": "target-a"},
            "children": if destination == "a" { vec![button.clone()] } else { vec![] },
        }),
        json!({
            "tag": "div",
            "attributes": {"data-id": "b", "id": "target-b"},
            "children": [],
        }),
    ];
    if destination == "disabled" {
        children.push(button);
    }
    children.push(json!({
        "tag": "i",
        "attributes": {"data-id": "tail"},
        "children": ["tail"],
    }));
    json!({
        "tree": [{"tag": "main", "attributes": {"data-id": "root"}, "children": children}],
        "events": events,
        "identities": if destination == "a" {
            json!([["root", 0], ["a", 1], ["button", 3], ["b", 2], ["tail", 4]])
        } else {
            json!([["root", 0], ["a", 1], ["b", 2], ["button", 3], ["tail", 4]])
        },
    })
}

#[test]
fn computed_disabled_key_removes_stale_props_and_restores_them() {
    let allocator = Allocator::new();
    let options = VaporCompilerOptions {
        prefix_identifiers: true,
        ..Default::default()
    };
    let walks = WalkCounts::snapshot();
    let parses = expr_parse_probe::expr_parse_count();
    let native = compile_vapor(&allocator, SOURCE, options.clone());
    assert!(
        native.error_messages.is_empty(),
        "native: {:?}",
        native.error_messages
    );
    assert_eq!(
        WalkCounts::snapshot().since(walks).total_walks(),
        0,
        "computed Teleport prop key must stay on native L3"
    );
    assert_eq!(
        expr_parse_probe::expr_parse_count() - parses,
        0,
        "computed Teleport prop key must retain its expression tree"
    );

    let retained = compile_vapor(
        &allocator,
        SOURCE,
        VaporCompilerOptions {
            davinci_retained_lane: true,
            ..options
        },
    );
    assert!(
        retained.error_messages.is_empty(),
        "retained: {:?}",
        retained.error_messages
    );

    let context = json!({"flag": "disabled", "enabled": true, "label": "A"});
    let steps = json!([
        {"click": "button"},
        {"patch": {"flag": "inert", "label": "B"}},
        {"click": "button"},
        {"patch": {"flag": "disabled", "label": "C"}},
        {"click": "button"},
    ]);
    let expected = vec![
        view("disabled", "A", json!([])),
        view("disabled", "A", json!(["A"])),
        view("a", "B", json!(["A"])),
        view("a", "B", json!(["A", "B"])),
        view("disabled", "C", json!(["A", "B"])),
        view("disabled", "C", json!(["A", "B", "C"])),
        json!({"tree": [], "events": ["A", "B", "C"], "identities": []}),
    ];
    for (lane, code) in [("native", native.code), ("retained", retained.code)] {
        let actual = trace::trace(
            "davinci-mounted-trace.mjs",
            json!({
                "backend": "vapor",
                "code": code,
                "context": context.clone(),
                "steps": steps.clone(),
                "identities": true,
            }),
        );
        assert_eq!(actual, expected, "{lane} trace");
    }
    let official = trace::trace(
        "davinci-upstream-vapor-trace.mjs",
        json!({"source": SOURCE, "context": context, "steps": steps}),
    );
    assert_eq!(official, expected, "official compiler-vapor trace");
}
