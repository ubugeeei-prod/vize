//! P3-4 supplied-slot reference cases and their S3 provenance.
//!
//! Each case mounts an authored template under a caller that supplies named or
//! default slots, either as static text or as one displayed slot prop, while
//! reactive patches change props, branches and loop items. Rust lowers every
//! case and commits the exact graph/value Folios. The Lean reference computes
//! the expected observations and both mounted Vue runtimes must reproduce them.
//! Regenerate with `VIZE_UPDATE_SLOT_REFERENCE=1`, then
//! `lake exe impetoRef --write-slot-reference`.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types
)]

use serde_json::{Value, json};
use std::path::Path;
use vize_davinci::folio::{Folio, FolioMode};
use vize_s0::Allocator;
use vize_s3::folio::S3Folio;
use vize_s3::values_folio::S3ValuesFolio;
use vize_s3::verify::verify;

fn patch(key: &str, value: Value) -> Value {
    json!({ "patch": { key: value } })
}

const PROP_HEADER: &str =
    r#"<main><slot name="header" :item="label" title="t">fallback</slot><p>{{ label }}</p></main>"#;
const DEFAULT_AND_OTHER: &str =
    r#"<main><slot>fallback</slot><slot name="other">other {{ label }}</slot></main>"#;

fn cases() -> Vec<(&'static str, &'static str, Value, Value, Vec<Value>)> {
    let items = |rows: &[(&str, &str)]| {
        json!(
            rows.iter()
                .map(|(id, label)| json!({ "id": id, "label": label }))
                .collect::<Vec<_>>()
        )
    };
    vec![
        (
            "named-text",
            r#"<main><slot name="header">fallback {{ label }}</slot><p>{{ label }}</p></main>"#,
            json!({ "header": { "text": "Supplied" } }),
            json!({ "label": "one" }),
            vec![patch("label", json!("two"))],
        ),
        (
            "named-prop",
            PROP_HEADER,
            json!({ "header": { "prop": "item" } }),
            json!({ "label": "one" }),
            vec![patch("label", json!("two")), patch("label", json!("雪"))],
        ),
        (
            "static-prop",
            PROP_HEADER,
            json!({ "header": { "prop": "title" } }),
            json!({ "label": "one" }),
            vec![patch("label", json!("two"))],
        ),
        (
            "default-supplied",
            DEFAULT_AND_OTHER,
            json!({ "default": { "text": "Default" } }),
            json!({ "label": "one" }),
            vec![patch("label", json!("two"))],
        ),
        (
            "none-supplied",
            DEFAULT_AND_OTHER,
            json!({}),
            json!({ "label": "one" }),
            vec![patch("label", json!("two"))],
        ),
        (
            "branch-prop",
            r#"<main><section v-if="ready"><slot name="header" :item="label">fallback</slot></section><p v-else>waiting</p></main>"#,
            json!({ "header": { "prop": "item" } }),
            json!({ "ready": true, "label": "one" }),
            vec![
                patch("label", json!("two")),
                patch("ready", json!(false)),
                patch("label", json!("three")),
                patch("ready", json!(true)),
            ],
        ),
        (
            "loop-prop",
            r#"<main><div v-for="item in items" :key="item.id" :data-id="item.id"><slot name="row" :item="item.label">none</slot></div></main>"#,
            json!({ "row": { "prop": "item" } }),
            json!({ "items": items(&[("a", "A"), ("b", "B")]) }),
            vec![
                patch("items", items(&[("b", "B"), ("a", "A")])),
                patch("items", items(&[("b", "B2"), ("c", "C")])),
                patch("items", items(&[])),
            ],
        ),
        (
            "computed-prop",
            r#"<main><slot name="header" :item="label + '!' + count * 2">fallback</slot></main>"#,
            json!({ "header": { "prop": "item" } }),
            json!({ "label": "n", "count": 1 }),
            vec![patch("count", json!(5)), patch("label", json!("m"))],
        ),
    ]
}

fn lowered(source: &str) -> (String, String) {
    let allocator = Allocator::default();
    let (tree, errors) = vize_s1::parse(&allocator, source);
    assert!(errors.is_empty(), "{errors:?}");
    let s2 = vize_s1_to_s2::lower(&allocator, &tree, &errors);
    assert!(s2.diagnostics.is_empty(), "{:?}", s2.diagnostics);
    let lowered = vize_s2_to_s3::lower(&allocator, &s2.root);
    assert_eq!(verify(&lowered.program), [], "{source}");
    (
        S3Folio::of(&lowered.program)
            .print_to_string(FolioMode::Full)
            .to_string(),
        S3ValuesFolio::of(&lowered.program)
            .print_to_string(FolioMode::Full)
            .to_string(),
    )
}

fn check_lines(path: &Path, lines: &[Value]) {
    let actual: String = lines.iter().map(|line| format!("{line}\n")).collect();
    if std::env::var("VIZE_UPDATE_SLOT_REFERENCE").as_deref() == Ok("1") {
        std::fs::write(path, &actual).unwrap();
    }
    assert_eq!(
        actual,
        std::fs::read_to_string(path).unwrap(),
        "{}",
        path.display()
    );
}

#[test]
fn slot_reference_cases_are_rust_lowered() {
    let mut rows = Vec::new();
    let mut graphs = Vec::new();
    for (name, template, slots, context, steps) in cases() {
        let (graph, values) = lowered(template);
        rows.push(json!({
            "name": name,
            "template": template,
            "scenario": { "context": context, "slots": slots, "steps": steps },
        }));
        graphs.push(json!({ "name": name, "graph": graph, "values": values }));
    }
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/formal/impeto/fixtures");
    check_lines(&fixtures.join("slot-reference.cases.jsonl"), &rows);
    check_lines(&fixtures.join("slot-reference.lowered.jsonl"), &graphs);
}
