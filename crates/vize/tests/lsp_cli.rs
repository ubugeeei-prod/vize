use std::path::Path;

use serde_json::json;
use vize_s0::{corsa_resolver::discover_corsa_in_ancestors, cstr};

#[path = "support/lsp_process.rs"]
mod lsp_process;

use lsp_process::{LspProcess, file_uri};

#[test]
fn lsp_exit_notification_terminates_process_while_stdin_stays_open() {
    let project = tempfile::tempdir().unwrap();
    let root_uri = file_uri(project.path());
    let mut lsp = LspProcess::spawn(project.path());

    lsp.send(json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "processId": null,
            "rootUri": root_uri,
            "capabilities": {},
            "initializationOptions": {
                "lint": false,
                "typecheck": false,
                "ecosystem": false
            }
        }
    }));
    let initialize = lsp.recv_response(1);
    assert!(initialize["result"].is_object(), "{initialize:#}");

    lsp.send(json!({
        "jsonrpc": "2.0",
        "method": "initialized",
        "params": {}
    }));
    lsp.send(json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "shutdown"
    }));
    let shutdown = lsp.recv_response(2);
    assert!(shutdown["result"].is_null(), "{shutdown:#}");

    lsp.send(json!({
        "jsonrpc": "2.0",
        "method": "exit"
    }));
    let status = lsp.wait_for_exit();
    assert!(status.success(), "LSP exited with {status}");
}

#[test]
fn lsp_corsa_smoke_publishes_diagnostics_and_hover() {
    let workspace_root = workspace_root();
    let Some(corsa_path) = discover_corsa_in_ancestors(workspace_root) else {
        eprintln!("skipping LSP Corsa smoke: TypeScript 7 Corsa runtime is unavailable");
        return;
    };
    let project = create_lsp_project(workspace_root, &corsa_path);
    let project_root = project.path();
    let app_path = project_root.join("src/App.vue");
    let source = std::fs::read_to_string(&app_path).unwrap();
    let (hover_line, hover_character) = lsp_position(&source, "count.toFixed");
    let root_uri = file_uri(project_root);
    let app_uri = file_uri(&app_path);
    let mut lsp = LspProcess::spawn(project_root);

    lsp.send(json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "processId": null,
            "rootUri": root_uri,
            "capabilities": {},
            "initializationOptions": {
                "lint": true,
                "typecheck": true,
                "hover": true
            }
        }
    }));
    let initialize = lsp.recv_response(1);
    if !initialize["result"]["capabilities"]["hoverProvider"]
        .as_bool()
        .unwrap_or(false)
    {
        lsp.fail(cstr!(
            "LSP initialize response did not enable hover:\n{initialize:#}"
        ));
    }

    lsp.send(json!({
        "jsonrpc": "2.0",
        "method": "initialized",
        "params": {}
    }));
    lsp.send(json!({
        "jsonrpc": "2.0",
        "method": "textDocument/didOpen",
        "params": {
            "textDocument": {
                "uri": app_uri,
                "languageId": "vue",
                "version": 1,
                "text": source
            }
        }
    }));

    // `didOpen` publishes one complete diagnostic set after the synchronous
    // lint pass and the awaited Corsa pass. Treat that publication as the
    // terminal result: waiting for a second message turns an actionable empty
    // result into a misleading timeout and used to hide the server's stderr.
    let publication = lsp.recv_matching(|message| {
        message["method"].as_str() == Some("textDocument/publishDiagnostics")
            && message["params"]["uri"].as_str() == Some(app_uri.as_str())
            && message["params"]["version"].as_i64() == Some(1)
    });
    let has_expected_diagnostic =
        publication["params"]["diagnostics"]
            .as_array()
            .is_some_and(|diagnostics| {
                diagnostics.iter().any(|diagnostic| {
                    diagnostic["message"].as_str().is_some_and(|message| {
                        message.contains("number") || message.contains("TS2322")
                    })
                })
            });
    if !has_expected_diagnostic {
        lsp.fail(cstr!(
            "LSP published a terminal diagnostic set without the expected type error:\n{publication:#}"
        ));
    }

    lsp.send(json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "textDocument/hover",
        "params": {
            "textDocument": { "uri": app_uri },
            "position": { "line": hover_line, "character": hover_character }
        }
    }));
    let hover = lsp.recv_response(2);
    if hover["result"].is_null() {
        lsp.fail(cstr!("LSP returned no hover result:\n{hover:#}"));
    }
}

fn lsp_position(source: &str, needle: &str) -> (u32, u32) {
    let offset = source
        .find(needle)
        .unwrap_or_else(|| panic!("LSP fixture does not contain {needle:?}"));
    let prefix = &source[..offset];
    let line = prefix.bytes().filter(|byte| *byte == b'\n').count();
    let line_start = prefix.rfind('\n').map_or(0, |index| index + 1);
    let character = prefix[line_start..].encode_utf16().count();
    (
        u32::try_from(line).expect("fixture line fits in u32"),
        u32::try_from(character).expect("fixture character fits in u32"),
    )
}

#[test]
fn lsp_position_counts_utf16_code_units() {
    assert_eq!(lsp_position("😀 count", "count"), (0, 3));
}

fn create_lsp_project(workspace_root: &Path, corsa_path: &Path) -> tempfile::TempDir {
    let cases_root = workspace_root.join("target/vize-tests/tests");
    std::fs::create_dir_all(&cases_root).unwrap();
    let project = tempfile::Builder::new()
        .prefix("lsp-corsa-smoke-")
        .tempdir_in(cases_root)
        .unwrap();
    let project_root = project.path();
    std::fs::create_dir_all(project_root.join("src")).unwrap();
    std::fs::write(
        project_root.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true
  },
  "include": ["src/**/*"]
}"#,
    )
    .unwrap();
    std::fs::write(
        project_root.join("src/App.vue"),
        r#"<script setup lang="ts">
const count: number = 'oops'
</script>

<template>
  <div>{{ count.toFixed(1) }}</div>
</template>
"#,
    )
    .unwrap();
    let config = json!({
        "typeChecker": {
            "corsaPath": corsa_path,
        },
        "lsp": {
            "lint": true,
            "typecheck": true,
            "hover": true,
        },
    });
    std::fs::write(
        project_root.join("vize.config.json"),
        serde_json::to_vec_pretty(&config).unwrap(),
    )
    .unwrap();
    project
}

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root should exist")
}
