use std::path::Path;

use serde_json::json;
use vize_s0::{corsa_resolver::discover_corsa_in_ancestors, cstr};

#[path = "support/lsp_process.rs"]
mod lsp_process;

use lsp_process::{LspProcess, file_uri};

#[test]
fn lsp_corsa_resolves_dotted_basename_relative_imports() {
    let workspace_root = workspace_root();
    let Some(corsa_path) = discover_corsa_in_ancestors(workspace_root) else {
        eprintln!(
            "skipping LSP Corsa dotted basename regression: TypeScript 7 Corsa runtime is unavailable"
        );
        return;
    };
    let project = create_project(workspace_root, &corsa_path);
    let project_root = project.path();
    let app_path = project_root.join("src/a.vue");
    let source = std::fs::read_to_string(&app_path).unwrap();
    let app_uri = file_uri(&app_path);
    let mut lsp = LspProcess::spawn(project_root);

    initialize(&mut lsp, project_root);
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

    let publication = lsp.recv_matching(|message| {
        message["method"].as_str() == Some("textDocument/publishDiagnostics")
            && message["params"]["uri"].as_str() == Some(app_uri.as_str())
            && message["params"]["version"].as_i64() == Some(1)
    });
    let diagnostics = publication["params"]["diagnostics"]
        .as_array()
        .expect("diagnostics array");
    if diagnostics.iter().any(|diagnostic| {
        diagnostic["code"].as_i64() == Some(2307)
            || diagnostic["message"]
                .as_str()
                .is_some_and(|message| message.contains("Cannot find module './x.use'"))
    }) || !diagnostics.is_empty()
    {
        lsp.fail(cstr!(
            "LSP dotted basename import must not publish diagnostics:\n{publication:#}"
        ));
    }

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

fn initialize(lsp: &mut LspProcess, project_root: &Path) {
    lsp.send(json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "processId": null,
            "rootUri": file_uri(project_root),
            "capabilities": {},
            "initializationOptions": {
                "lint": false,
                "typecheck": true,
                "ecosystem": false
            }
        }
    }));
    let initialize = lsp.recv_response(1);
    assert!(initialize["result"].is_object(), "{initialize:#}");
}

fn create_project(workspace_root: &Path, corsa_path: &Path) -> tempfile::TempDir {
    let cases_root = workspace_root.join("target/vize-tests/tests");
    std::fs::create_dir_all(&cases_root).unwrap();
    let project = tempfile::Builder::new()
        .prefix("lsp-corsa-dotted-basename-")
        .tempdir_in(cases_root)
        .unwrap();
    let project_root = project.path();
    std::fs::create_dir_all(project_root.join("src")).unwrap();
    write_vue_package(project_root);
    write_project_config(project_root, corsa_path);
    std::fs::write(
        project_root.join("src/x.use.ts"),
        r#"export type ValidationRule = (v: unknown) => true | string;
export const useX = () => 1;
"#,
    )
    .unwrap();
    std::fs::write(
        project_root.join("src/a.vue"),
        r#"<script setup lang="ts">
import { useX, type ValidationRule } from "./x.use";
const r: ValidationRule = () => true;
console.log(useX(), r);
</script>

<template><div /></template>
"#,
    )
    .unwrap();
    project
}

fn write_vue_package(project_root: &Path) {
    let vue_dir = project_root.join("node_modules/vue");
    std::fs::create_dir_all(&vue_dir).unwrap();
    std::fs::write(
        vue_dir.join("package.json"),
        r#"{"name":"vue","version":"3.0.0","types":"index.d.ts"}"#,
    )
    .unwrap();
    std::fs::write(
        vue_dir.join("index.d.ts"),
        r#"export type DefineComponent<P = any> = { new(): { $props: P } };
export interface ComponentPublicInstance {
  $attrs: Record<string, unknown>;
  $slots: Record<string, unknown>;
  $refs: Record<string, unknown>;
  $emit: (...args: unknown[]) => void;
}
"#,
    )
    .unwrap();
}

fn write_project_config(project_root: &Path, corsa_path: &Path) {
    std::fs::write(
        project_root.join("tsconfig.json"),
        r#"{"compilerOptions":{"strict":true,"moduleResolution":"bundler","module":"ESNext","target":"ESNext","noEmit":true},"include":["src/**/*"]}"#,
    )
    .unwrap();
    let config = json!({
        "typeChecker": {
            "corsaPath": corsa_path,
        },
        "lsp": {
            "lint": false,
            "typecheck": true,
            "ecosystem": false,
        },
    });
    std::fs::write(
        project_root.join("vize.config.json"),
        serde_json::to_vec_pretty(&config).unwrap(),
    )
    .unwrap();
}

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root should exist")
}
