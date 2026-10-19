//! Vapor parity ratchet against `@vue/compiler-vapor` (#8859).
//!
//! Every case in `tests/_fixtures/vapor-parity` is compiled here and handed,
//! with its source, to `tests/tooling/support/vapor-parity/run.ts`. The runner
//! mounts it, and hydrates the official server render of it, with Vize's and
//! the official output under the same Vue release and reports the cases that
//! behave differently. Those must equal `known-differences.json`: a new
//! difference is a regression, and a case that starts matching must leave the
//! list, so the list only shrinks. `VIZE_VAPOR_PARITY_UPDATE=1` rewrites the
//! list when nothing regressed; `=reset` rewrites it unconditionally, only for
//! a new Vue release or a new corpus.

#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "tests assert by panicking and read std-string JSON fixtures"
)]

use serde_json::{Value, json};
use std::{
    collections::BTreeSet,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};
use vize_atelier_vapor::{VaporCompilerOptions, compile_vapor};
use vize_carton::Allocator;

const CORPORA: [&str; 3] = [
    "lane-corpus",
    "upstream-compiler-vapor",
    "upstream-runtime-vapor-hydration",
];

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/_fixtures/vapor-parity")
}

fn read_json(path: &Path) -> Value {
    let text = std::fs::read_to_string(path).unwrap_or_else(|error| panic!("{path:?}: {error}"));
    serde_json::from_str(&text).unwrap_or_else(|error| panic!("{path:?}: {error}"))
}

fn compiled_corpus(name: &str) -> Value {
    let corpus = read_json(&fixtures().join(format!("{name}.json")));
    let cases: Vec<Value> = corpus["cases"]
        .as_array()
        .expect("corpus cases")
        .iter()
        .map(|case| {
            let source = case["source"].as_str().expect("case source");
            let allocator = Allocator::new();
            let result = compile_vapor(
                &allocator,
                source,
                VaporCompilerOptions {
                    prefix_identifiers: true,
                    ..VaporCompilerOptions::default()
                },
            );
            json!({
                "id": case["id"],
                "source": source,
                "code": result.code.as_str(),
                "errors": result.error_messages.iter().map(|m| m.as_str()).collect::<Vec<_>>(),
            })
        })
        .collect();
    json!({ "name": name, "cases": cases })
}

fn run(input: &Value) -> Value {
    let runner = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/tooling/support/vapor-parity/run.ts");
    let mut child = Command::new("node")
        .arg(runner)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("start the Vapor parity runner (install workspace JS dependencies first)");
    child
        .stdin
        .take()
        .expect("runner stdin")
        .write_all(input.to_string().as_bytes())
        .expect("send compiled corpora");
    let output = child.wait_with_output().expect("runner exits");
    assert!(
        output.status.success(),
        "runner failed with {}:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|error| panic!("{error}: {}", String::from_utf8_lossy(&output.stdout)))
}

fn ids(value: &Value) -> BTreeSet<String> {
    value
        .as_array()
        .map(|ids| {
            ids.iter()
                .filter_map(|id| id.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

#[test]
fn vapor_parity_known_differences_only_shrink() {
    let corpora: Vec<Value> = CORPORA.iter().map(|name| compiled_corpus(name)).collect();
    let receipt = run(&json!({ "corpora": corpora }));
    let known_path = fixtures().join("known-differences.json");
    let known = read_json(&known_path);
    assert_eq!(
        receipt["vue"], known["vue"],
        "the ratchet is pinned to one Vue release"
    );

    let mut problems = Vec::new();
    let mut updated = serde_json::Map::new();
    for corpus in receipt["corpora"].as_array().expect("receipt corpora") {
        let name = corpus["name"].as_str().expect("corpus name");
        let mut modes = serde_json::Map::new();
        for mode in ["mount", "hydrate"] {
            let actual = ids(&corpus[mode]["different"]);
            let expected = ids(&known["corpora"][name][mode]);
            let cases = corpus[mode]["cases"].as_u64().unwrap_or_default();
            eprintln!(
                "{name} {mode}: {}/{cases} match upstream",
                cases - actual.len() as u64
            );
            let regressed: Vec<_> = actual.difference(&expected).collect();
            let fixed: Vec<_> = expected.difference(&actual).collect();
            if !regressed.is_empty() {
                problems.push(format!(
                    "{name} {mode} now differs from upstream: {regressed:?}"
                ));
            }
            if !fixed.is_empty() {
                problems.push(format!(
                    "{name} {mode} now matches upstream; remove from known-differences.json \
                     (or rerun with VIZE_VAPOR_PARITY_UPDATE=1): {fixed:?}"
                ));
            }
            modes.insert(mode.into(), json!(actual));
        }
        updated.insert(name.into(), Value::Object(modes));
    }
    let regressed = problems
        .iter()
        .any(|problem| problem.contains("now differs"));
    let update = std::env::var("VIZE_VAPOR_PARITY_UPDATE").ok();
    if update.as_deref() == Some("reset") || (update.is_some() && !regressed) {
        let next = json!({ "vue": receipt["vue"], "corpora": updated });
        let text = serde_json::to_string_pretty(&next).expect("serialize ratchet") + "\n";
        std::fs::write(&known_path, text).expect("write known-differences.json");
        return;
    }
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}
