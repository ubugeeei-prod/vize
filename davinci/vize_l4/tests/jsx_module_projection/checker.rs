use std::collections::BTreeSet;
use std::io::Write;
use std::process::{Command, Stdio};

use super::support;
use vize_l0::Allocator;
use vize_l4::targets::ts::{HELPERS, LinkMapping, TsDocument, project_jsx_module};

/// Names in generated text (outside verbatim links) spelled like an ambient
/// helper: `__vize…` values and `$Upper…` types.
fn written_helpers(projected: &TsDocument) -> Vec<String> {
    let text = projected.document().as_str();
    let mut generated = String::with_capacity(text.len());
    let mut cursor = 0;
    for (link, fact) in projected
        .document()
        .links()
        .iter()
        .zip(projected.link_facts())
    {
        if fact.mapping == LinkMapping::Verbatim {
            generated.push_str(&text[cursor..link.generated.start as usize]);
            generated.push(' ');
            cursor = link.generated.end as usize;
        }
    }
    generated.push_str(&text[cursor..]);
    generated
        .split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_' || ch == '$'))
        .filter(|word| {
            word.starts_with("__vize")
                || word
                    .strip_prefix('$')
                    .is_some_and(|rest| rest.starts_with(|ch: char| ch.is_ascii_uppercase()))
        })
        .map(String::from)
        .collect()
}

#[test]
fn real_typescript_diagnostics_map_to_exact_authored_spans() {
    let pack: serde_json::Value =
        serde_json::from_str(include_str!("../fixtures/jsx-module-checker.json")).unwrap();
    let cases = pack["cases"].as_array().unwrap();
    let arena = Allocator::default();
    let mut projected = Vec::new();
    let mut inputs = Vec::new();
    let mut written = BTreeSet::new();
    for case in cases {
        let source = case["source"].as_str().unwrap();
        let module = support::lower(&arena, source, case["kind"] == "tsx").unwrap();
        let document = project_jsx_module(&module.program, &module.roots).unwrap();
        written.extend(written_helpers(&document));
        inputs
            .push(serde_json::json!({"id": case["id"], "projected": document.document().as_str()}));
        projected.push((source, document));
    }
    // The table lists exactly the helpers the documents write.
    let mut table: Vec<&str> = HELPERS.to_vec();
    table.sort_unstable();
    assert_eq!(
        written.iter().map(String::as_str).collect::<Vec<_>>(),
        table
    );
    let mut child = Command::new("node")
        .args([
            "--input-type=module",
            "-e",
            include_str!("../../../../tests/tooling/support/native-jsx-module-typescript.ts"),
        ])
        .current_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(
            &serde_json::to_vec(&serde_json::json!({"cases": inputs, "helpers": HELPERS})).unwrap(),
        )
        .unwrap();
    let result = child.wait_with_output().unwrap();
    assert!(
        result.status.success(),
        "{}",
        std::str::from_utf8(&result.stderr).unwrap_or("non-UTF8 stderr")
    );
    let evidence: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    if let Ok(path) = std::env::var("VIZE_L4_JSX_MODULE_CHECK_CAPTURE") {
        std::fs::write(path, serde_json::to_vec_pretty(&evidence).unwrap()).unwrap();
    }
    assert_eq!(evidence["version"], "6.0.3");
    // The target names helpers; only the published ambient set declares them.
    assert_eq!(
        evidence["helpers"],
        serde_json::json!(
            HELPERS
                .iter()
                .map(|name| serde_json::json!({
                    "name": name,
                    "declarations": ["npm/cli/virtual-ts-globals.d.ts"],
                }))
                .collect::<Vec<_>>()
        )
    );
    let checked = evidence["cases"].as_array().unwrap();
    assert_eq!(checked.len(), cases.len());
    let mut mapped = Vec::new();
    for ((case, result), (source, document)) in cases.iter().zip(checked).zip(&projected) {
        assert_eq!(case["id"], result["id"]);
        let diagnostics: Vec<serde_json::Value> = result["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .map(|diagnostic| {
                let start = u32::try_from(diagnostic["start"].as_u64().unwrap()).unwrap();
                let length = u32::try_from(diagnostic["length"].as_u64().unwrap()).unwrap();
                // Every diagnostic maps: the unmapped count is zero.
                let authored = document.map_utf16(start, length).unwrap();
                serde_json::json!({
                    "code": diagnostic["code"],
                    "category": diagnostic["category"],
                    "message": diagnostic["message"],
                    "authored": &source[authored.start as usize..authored.end as usize],
                    "at": authored.start,
                })
            })
            .collect();
        mapped.push(serde_json::json!({"id": case["id"], "diagnostics": diagnostics}));
    }
    let expected: Vec<serde_json::Value> = cases
        .iter()
        .map(|case| serde_json::json!({"id": case["id"], "diagnostics": case["diagnostics"]}))
        .collect();
    assert_eq!(mapped, expected);
}
