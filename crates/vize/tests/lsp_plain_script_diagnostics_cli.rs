#![cfg(test)]
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_macros, reason = "fixtures use std strings")]
#![expect(clippy::string_slice, reason = "tests assert by panicking")]
use serde_json::{Value, json};
use std::path::Path;
use vize_carton::corsa_resolver::discover_corsa_in_ancestors;

#[path = "support/lsp_process.rs"]
mod lsp_process;
use lsp_process::{LspProcess, file_uri};

const SOURCE: &str =
    include_str!("../../../tests/_fixtures/differential/lsp/plain-script-diagnostics/plain.ts.txt");
const CONFIG: &str = include_str!(
    "../../../tests/_fixtures/differential/lsp/plain-script-diagnostics/tsconfig.json"
);

#[test]
fn complete_original_default_publications_follow_extensions_despite_language_ids() {
    let mut mismatches = Vec::new();
    for (extension, language_id) in [
        ("ts", "typescript"),
        ("mts", "typescript"),
        ("ts", "typescriptreact"),
        ("tsx", "typescriptreact"),
        ("tsx", "typescript"),
    ] {
        let mut fixture = Fixture::new(extension, SOURCE, false);
        fixture.open(SOURCE, language_id, json!([]));
        mismatches.extend(fixture.shutdown());
    }
    // The same bytes really remain malformed SFC input on a Vue URI.
    let mut fixture = Fixture::new("vue", SOURCE, false);
    fixture.open(
        SOURCE,
        "typescript",
        json!([{
            "range": { "start": { "line": 0, "character": 26 }, "end": { "line": 1, "character": 0 } },
            "severity": 1, "source": "vize/sfc",
            "message": "Malformed <string> block: the closing tag is missing."
        }]),
    );
    mismatches.extend(fixture.shutdown());
    assert!(mismatches.is_empty(), "{mismatches:#?}");
}

#[test]
fn native_ts_diagnostics_survive_generics_guards_utf16_unsaved_errors_and_repairs() {
    let mut mismatches = Vec::new();
    for newline in ["\n", "\r\n"] {
        for extension in ["ts", "mts", "cts"] {
            for language_id in ["typescript", "typescriptreact", "vue"] {
                let original = SOURCE.replace('\n', newline);
                let mut fixture = Fixture::new(extension, &original, true);
                fixture.open(&original, language_id, json!([]));
                let changed = format!(
                    "// 😀{newline}{original}/* 😀 */ const value: number = 'wrong';{newline}"
                );
                let start = position(&changed, changed.find("value:").unwrap());
                let end = json!({ "line": start["line"], "character": start["character"].as_u64().unwrap() + 5 });
                let hint = unused_hint("value", start.clone(), end.clone());
                let expected = json!([{
                    "range": { "start": start, "end": end }, "severity": 1,
                    "code": 2322, "source": "vize/types",
                    "message": "Type 'string' is not assignable to type 'number'."
                }, hint.clone()]);
                fixture.change(&changed, 2, expected.clone());
                let repaired = changed.replace("'wrong'", "1");
                fixture.change(&repaired, 3, json!([hint]));
                fixture.change(&changed, 4, expected);
                let guard = format!(
                    "{original}function isObject(x: unknown): x is Record<string, unknown> {{ return typeof x === 'object' && x !== null; }}{newline}"
                );
                let guard_start = position(&guard, guard.find("isObject(").unwrap());
                let guard_end = json!({ "line": guard_start["line"], "character": guard_start["character"].as_u64().unwrap() + 8 });
                fixture.change(
                    &guard,
                    5,
                    json!([unused_hint("isObject", guard_start, guard_end)]),
                );
                fixture.change(&original, 6, json!([]));
                mismatches.extend(fixture.shutdown());
            }
        }
    }
    assert!(mismatches.is_empty(), "{mismatches:#?}");
}

#[test]
fn javascript_variants_keep_plain_script_ownership_with_misleading_vue_language_id() {
    let mut mismatches = Vec::new();
    let source = "export const map = new Map();\nconst text = '<string>';\n";
    for extension in ["js", "mjs", "cjs"] {
        let mut fixture = Fixture::new(extension, source, true);
        let hint = unused_hint(
            "text",
            json!({ "line": 1, "character": 6 }),
            json!({ "line": 1, "character": 10 }),
        );
        fixture.open(source, "vue", json!([hint.clone()]));
        fixture.change("export const map = new Map();\n", 2, json!([]));
        fixture.change(source, 3, json!([hint]));
        mismatches.extend(fixture.shutdown());
    }
    assert!(mismatches.is_empty(), "{mismatches:#?}");
}

struct Fixture {
    _project: tempfile::TempDir,
    uri: String,
    process: LspProcess,
    mismatches: Vec<Value>,
}

impl Fixture {
    fn new(extension: &str, source: &str, typed: bool) -> Self {
        let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap();
        let cases = workspace.join("target/vize-tests/tests");
        std::fs::create_dir_all(&cases).unwrap();
        let project = tempfile::Builder::new()
            .prefix("lsp-plain-script-")
            .tempdir_in(cases)
            .unwrap();
        if typed {
            let runtime = discover_corsa_in_ancestors(workspace)
                .expect("plain script tests require the actual TypeScript native runtime");
            std::fs::write(project.path().join("tsconfig.json"), CONFIG).unwrap();
            std::fs::write(
                project.path().join("vize.config.json"),
                serde_json::to_vec(&json!({
                    "typeChecker": { "corsaPath": runtime },
                    "lsp": { "lint": true, "typecheck": true }
                }))
                .unwrap(),
            )
            .unwrap();
        }
        let path = project.path().join(format!("plain.{extension}"));
        std::fs::write(&path, source).unwrap();
        let uri = file_uri(&path).to_string();
        let mut process = LspProcess::spawn(project.path());
        process.send(
            json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {
                "processId": null, "rootUri": file_uri(project.path()), "capabilities": {}
            }}),
        );
        let response = process.recv_response(1);
        assert!(response.get("error").is_none(), "{response:#}");
        assert!(response["result"].is_object());
        process.send(json!({ "jsonrpc": "2.0", "method": "initialized", "params": {} }));
        Self {
            _project: project,
            uri,
            process,
            mismatches: Vec::new(),
        }
    }

    fn open(&mut self, source: &str, language_id: &str, expected: Value) {
        self.process.send(json!({ "jsonrpc": "2.0", "method": "textDocument/didOpen", "params": {
            "textDocument": { "uri": self.uri, "languageId": language_id, "version": 1, "text": source }
        }}));
        self.publication(1, expected);
    }

    fn change(&mut self, source: &str, version: i32, expected: Value) {
        self.process.send(json!({ "jsonrpc": "2.0", "method": "textDocument/didChange", "params": {
            "textDocument": { "uri": self.uri, "version": version }, "contentChanges": [{ "text": source }]
        }}));
        self.publication(version, expected);
    }

    fn publication(&mut self, version: i32, expected: Value) {
        let message = self.process.recv_matching(|message| {
            message["method"] == "textDocument/publishDiagnostics"
                && message["params"]["uri"] == self.uri
                && message["params"]["version"] == version
        });
        let expected = json!({ "jsonrpc": "2.0", "method": "textDocument/publishDiagnostics", "params": {
            "uri": self.uri, "version": version, "diagnostics": expected
        }});
        if message != expected {
            self.mismatches
                .push(json!({ "actual": message, "expected": expected }));
        }
    }

    fn shutdown(&mut self) -> Vec<Value> {
        self.process
            .send(json!({ "jsonrpc": "2.0", "id": 2, "method": "shutdown" }));
        assert_eq!(
            self.process.recv_response(2),
            json!({ "jsonrpc": "2.0", "id": 2, "result": null })
        );
        self.process
            .send(json!({ "jsonrpc": "2.0", "method": "exit" }));
        assert!(self.process.wait_for_exit().success());
        std::mem::take(&mut self.mismatches)
    }
}

fn position(source: &str, offset: usize) -> Value {
    let prefix = source[..offset].replace("\r\n", "\n").replace('\r', "\n");
    let line = prefix.bytes().filter(|&byte| byte == b'\n').count();
    let start = prefix.rfind('\n').map_or(0, |index| index + 1);
    json!({ "line": line, "character": prefix[start..].encode_utf16().count() })
}

fn unused_hint(name: &str, start: Value, end: Value) -> Value {
    json!({
        "range": { "start": start, "end": end }, "severity": 4,
        "code": 6133, "source": "vize/types",
        "message": format!("'{name}' is declared but its value is never read.")
    })
}
