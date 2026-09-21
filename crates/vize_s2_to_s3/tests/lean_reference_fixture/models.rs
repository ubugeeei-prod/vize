//! P3-4 native `v-model` reference cases and their S3 provenance.
//!
//! Each case is an authored template plus a scenario of user events and
//! reactive patches: IME composition, `.lazy`, `.trim`, `.number`, checkbox
//! booleans and arrays, radios, single/multiple/numeric selects, member paths
//! and controls recreated by `v-if`. Rust lowers every case and commits the
//! exact graph/value Folios. The Lean model reference computes the expected
//! observations and both mounted Vue runtimes must reproduce them. Regenerate
//! with `VIZE_UPDATE_MODEL_REFERENCE=1`, then
//! `lake exe impetoRef --write-model-reference`.

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

fn event(name: &str, selector: &str) -> Value {
    json!({ "event": name, "selector": selector })
}

fn with(mut step: Value, key: &str, value: Value) -> Value {
    step[key] = value;
    step
}

fn patch(key: &str, value: Value) -> Value {
    json!({ "patch": { key: value } })
}

fn cases() -> Vec<(&'static str, &'static str, Value, Vec<Value>)> {
    let input = |value: &str| with(event("input", "input"), "value", json!(value));
    vec![
        (
            "text-ime",
            r#"<section><input v-model="value"><p>{{ value }}</p></section>"#,
            json!({ "value": "initial" }),
            vec![
                input("typed"),
                event("compositionstart", "input"),
                input("にほん"),
                with(event("compositionend", "input"), "value", json!("日本")),
                patch("value", json!("external")),
                patch("value", Value::Null),
                event("change", "input"),
            ],
        ),
        (
            "text-lazy",
            r#"<section><input v-model.lazy="value"><p>{{ value }}</p></section>"#,
            json!({ "value": "initial" }),
            vec![
                input("pending"),
                with(event("change", "input"), "value", json!("committed")),
                patch("value", json!("external")),
            ],
        ),
        (
            "text-trim",
            r#"<section><input v-model.trim="value"><p>{{ '[' + value + ']' }}</p></section>"#,
            json!({ "value": "" }),
            vec![input("  hi  "), input(" hi"), event("change", "input")],
        ),
        (
            "text-trim-number",
            r#"<section><input v-model.trim.number="value"><p>{{ value }}</p></section>"#,
            json!({ "value": 0 }),
            vec![
                input(" 42 "),
                event("change", "input"),
                input("abc"),
                input("007"),
                input("-5"),
                patch("value", json!(12)),
            ],
        ),
        (
            "checkbox-array",
            r#"<section><input type="checkbox" value="b" v-model="values"><p>{{ values.length }}</p></section>"#,
            json!({ "values": ["a"] }),
            vec![
                with(event("change", "input"), "checked", json!(true)),
                with(event("change", "input"), "checked", json!(false)),
                patch("values", json!(["b"])),
                patch("values", json!(["x"])),
            ],
        ),
        (
            "checkbox-boolean",
            r#"<section><input type="checkbox" v-model="on"><p>{{ on ? 'yes' : 'no' }}</p></section>"#,
            json!({ "on": false }),
            vec![
                with(event("change", "input"), "checked", json!(true)),
                with(event("change", "input"), "checked", json!(false)),
                patch("on", json!(true)),
            ],
        ),
        (
            "radio-pair",
            r#"<section><input type="radio" value="x" v-model="pick"><input type="radio" value="y" v-model="pick"><p>{{ pick }}</p></section>"#,
            json!({ "pick": "x" }),
            vec![
                with(event("change", "input[value=y]"), "checked", json!(true)),
                patch("pick", json!("x")),
                patch("pick", json!("none")),
                with(event("change", "input[value=x]"), "checked", json!(true)),
            ],
        ),
        (
            "select-single",
            r#"<section><select v-model="choice"><option value="a">A</option><option value="b">B</option></select><p>{{ choice }}</p></section>"#,
            json!({ "choice": "a" }),
            vec![
                with(event("change", "select"), "value", json!("b")),
                patch("choice", json!("a")),
                patch("choice", json!("zzz")),
                patch("choice", json!("b")),
            ],
        ),
        (
            "select-multiple",
            r#"<section><select multiple v-model="picks"><option value="a">A</option><option value="b">B</option><option value="c">C</option></select><p>{{ picks.length }}</p></section>"#,
            json!({ "picks": ["a"] }),
            vec![
                with(
                    event("change", "select"),
                    "selectedValues",
                    json!(["a", "c"]),
                ),
                patch("picks", json!(["b"])),
                with(event("change", "select"), "selectedValues", json!([])),
            ],
        ),
        (
            "select-number",
            r#"<section><select v-model.number="n"><option value="1">One</option><option value="2">Two</option></select><p>{{ n + 1 }}</p></section>"#,
            json!({ "n": 1 }),
            vec![
                with(event("change", "select"), "value", json!("2")),
                patch("n", json!(1)),
            ],
        ),
        (
            "member-path",
            r#"<section><input v-model="form.value"><p>{{ form.value }}</p></section>"#,
            json!({ "form": { "value": "x" } }),
            vec![input("y"), patch("form", json!({ "value": "z" }))],
        ),
        (
            "recreated-control",
            r#"<section><input v-if="show" v-model="value"><p>{{ value }}</p></section>"#,
            json!({ "show": true, "value": "a" }),
            vec![
                input("typed"),
                patch("show", json!(false)),
                patch("value", json!("b")),
                patch("show", json!(true)),
            ],
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
    if std::env::var("VIZE_UPDATE_MODEL_REFERENCE").as_deref() == Ok("1") {
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
fn model_reference_cases_are_rust_lowered() {
    let mut rows = Vec::new();
    let mut graphs = Vec::new();
    for (name, template, context, steps) in cases() {
        let (graph, values) = lowered(template);
        rows.push(json!({
            "name": name,
            "template": template,
            "scenario": { "context": context, "steps": steps },
        }));
        graphs.push(json!({ "name": name, "graph": graph, "values": values }));
    }
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../formal/impeto/fixtures");
    check_lines(&fixtures.join("model-reference.cases.jsonl"), &rows);
    check_lines(&fixtures.join("model-reference.lowered.jsonl"), &graphs);
}
