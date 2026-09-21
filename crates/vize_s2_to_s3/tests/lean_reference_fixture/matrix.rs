//! P3-11 TS-29 matrix generator and S3 provenance bridge.
//!
//! The IVM matrix is the product of iteration sources (keyed and positional
//! arrays, objects and ranges), item bodies (non-linear text, per-item
//! conditional toggles, disabled/click buttons) and wrappers (open, or a
//! `v-if` guard that recreates the list). Each case is lowered here; its
//! authored template/scenario and exact graph/value Folios are committed as
//! JSON lines. The Lean reference computes the expected observations and both
//! mounted runtimes must reproduce them. Regenerate with
//! `VIZE_UPDATE_IVM_MATRIX=1`, then `lake exe impetoRef --write-ivm-matrix`.

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

struct Source {
    name: &'static str,
    head: &'static str,
    label: &'static str,
}

const SOURCES: [Source; 6] = [
    Source {
        name: "array-keyed",
        head: r#"v-for="(item, index) in items" :key="item.id" :data-id="item.id""#,
        label: "item.label",
    },
    Source {
        name: "array-positional",
        head: r#"v-for="(item, index) in items" :data-id="item.id""#,
        label: "item.label",
    },
    Source {
        name: "object-keyed",
        head: r#"v-for="(item, name, index) in table" :key="name" :data-id="name""#,
        label: "item.label",
    },
    Source {
        name: "object-positional",
        head: r#"v-for="(item, name, index) in table" :data-id="name""#,
        label: "item.label",
    },
    Source {
        name: "range-keyed",
        head: r#"v-for="(n, index) in count" :key="n" :data-id="'n' + n""#,
        label: "",
    },
    Source {
        name: "range-positional",
        head: r#"v-for="(n, index) in count" :data-id="'n' + n""#,
        label: "",
    },
];

fn body(source: &Source, body: &str) -> Option<String> {
    let range = source.label.is_empty();
    Some(match (body, range) {
        ("text", false) => "{{ item.label + ':' + item.qty * factor }}".to_string(),
        ("text", true) => "{{ n * factor + index }}".to_string(),
        ("toggle", false) => concat!(
            r#"<span v-if="item.qty > limit">{{ item.label }}!</span>"#,
            r#"<button v-else-if="item.on">{{ item.label + '?' }}</button>"#,
            r#"<span v-else>{{ item.qty % 2 === 0 ? 'even' : 'odd' }}</span>"#
        )
        .to_string(),
        ("toggle", true) => concat!(
            r#"<span v-if="n % 2 === 0">{{ n + ' even' }}</span>"#,
            r#"<span v-else>{{ n > limit ? 'big' : 'small' }}</span>"#
        )
        .to_string(),
        ("button", false) => format!(
            r#"<button :data-id="'btn-' + item.id" :disabled="!item.on" @click="record({})">{{{{ {} || 'none' }}}}</button>"#,
            source.label, source.label
        ),
        _ => return None,
    })
}

fn template(source: &Source, body: &str, wrapper: &str) -> String {
    let row = format!(r#"<div {} :title="index">{body}</div>"#, source.head);
    match wrapper {
        "open" => format!("<main>{row}<p>{{{{ 'total ' + total }}}}</p></main>"),
        _ => format!(
            r#"<main><section v-if="visible">{row}</section><p v-else>{{{{ 'hidden ' + total }}}}</p></main>"#
        ),
    }
}

fn record(id: &str, label: &str, qty: i64, on: bool) -> Value {
    json!({ "id": id, "label": label, "qty": qty, "on": on })
}

fn scenario(source: &Source, body: &str) -> Value {
    let (a, b, c, d, b2) = (
        ("a", "A", 1, true),
        ("b", "B", 4, false),
        ("c", "C", 7, true),
        ("d", "D", 2, false),
        ("b", "B", 10, true),
    );
    let items = |rows: &[(&str, &str, i64, bool)]| -> Value {
        if source.name.starts_with("array") {
            json!({ "items": rows.iter().map(|r| record(r.0, r.1, r.2, r.3)).collect::<Vec<_>>() })
        } else {
            let table: serde_json::Map<String, Value> = rows
                .iter()
                .map(|r| (r.0.to_string(), record(r.0, r.1, r.2, r.3)))
                .collect();
            json!({ "table": table })
        }
    };
    let clicks = |targets: &[&str]| -> Vec<Value> {
        if body == "button" {
            targets.iter().map(|t| json!({ "click": t })).collect()
        } else {
            Vec::new()
        }
    };
    let mut context = json!({ "factor": 2, "limit": 3, "visible": true, "total": 3 });
    let mut steps = vec![json!({ "patch": { "factor": 3 } })];
    if source.label.is_empty() {
        context["count"] = json!(3);
        for count in [5, 2] {
            steps.push(json!({ "patch": { "count": count } }));
        }
    } else {
        for (key, value) in items(&[a, b, c]).as_object().unwrap() {
            context[key] = value.clone();
        }
        let second = if source.name.starts_with("array") {
            items(&[c, a, b])
        } else {
            items(&[a, b2, c])
        };
        steps.push(json!({ "patch": second }));
        steps.extend(clicks(&["btn-a"]));
        let mut third = items(&[d, c, b2]);
        third["total"] = json!(4);
        steps.push(json!({ "patch": third }));
        steps.extend(clicks(&["btn-d", "btn-b"]));
    }
    steps.push(json!({ "patch": { "limit": 1 } }));
    steps.push(json!({ "patch": { "visible": false } }));
    steps.push(json!({ "patch": { "visible": true } }));
    let (empty, refill) = if source.label.is_empty() {
        (json!({ "count": 0 }), json!({ "count": 1 }))
    } else {
        (items(&[]), items(&[a]))
    };
    steps.push(json!({ "patch": empty }));
    steps.push(json!({ "patch": refill }));
    json!({ "context": context, "steps": steps })
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
    if std::env::var("VIZE_UPDATE_IVM_MATRIX").as_deref() == Ok("1") {
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
fn ivm_matrix_cases_are_rust_lowered() {
    let mut cases = Vec::new();
    let mut graphs = Vec::new();
    for source in &SOURCES {
        for body_name in ["text", "toggle", "button"] {
            let Some(body) = body(source, body_name) else {
                continue;
            };
            for wrapper in ["open", "guarded"] {
                let name = format!("{}-{body_name}-{wrapper}", source.name);
                let template = template(source, &body, wrapper);
                let (graph, values) = lowered(&template);
                cases.push(json!({
                    "name": name,
                    "template": template,
                    "scenario": scenario(source, body_name),
                }));
                graphs.push(json!({ "name": name, "graph": graph, "values": values }));
            }
        }
    }
    assert_eq!(cases.len(), 32);
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../formal/impeto/fixtures");
    check_lines(&fixtures.join("ivm-matrix.cases.jsonl"), &cases);
    check_lines(&fixtures.join("ivm-matrix.lowered.jsonl"), &graphs);
}
