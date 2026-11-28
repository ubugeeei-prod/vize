//! Project `GlobalComponents` declarations reach the shipped editor template checker.
#![expect(
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "tests assert by panicking"
)]

use super::DiagnosticService;
use super::editor_typecheck_fixture::{resolve_test_tsgo_binary, write_corsa_config};
use crate::server::ServerState;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use tower_lsp::lsp_types::Url;

fn workspace() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn locked_vue() -> Option<PathBuf> {
    [
        workspace().join("playground/node_modules/vue"),
        workspace().join("node_modules/vue"),
    ]
    .into_iter()
    .find_map(|path| path.canonicalize().ok())
}

struct Project {
    _dir: tempfile::TempDir,
    root: PathBuf,
    state: ServerState,
}

impl Project {
    fn new() -> Option<Self> {
        let corsa = resolve_test_tsgo_binary()?;
        let vue = locked_vue()?;
        let dir = tempfile::tempdir().expect("temp project");
        let root = dir.path().canonicalize().expect("physical root");
        let modules = root.join("node_modules");
        std::fs::create_dir(&modules).expect("node_modules");
        for (name, path) in [
            ("vue", vue.clone()),
            ("@vue", vue.parent().expect("vue scope").join("@vue")),
        ] {
            #[cfg(unix)]
            std::os::unix::fs::symlink(path, modules.join(name)).expect("link");
            #[cfg(windows)]
            std::os::windows::fs::symlink_dir(path, modules.join(name)).expect("link");
        }
        write_corsa_config(&root, &corsa);
        let state = ServerState::new();
        state.apply_lsp_initialization_options(Some(&json!({
            "lint": false,
            "ecosystem": false,
            "typecheck": true
        })));
        state.set_workspace_root(root.clone());
        state.load_workspace_config(&root);
        Some(Self {
            _dir: dir,
            root,
            state,
        })
    }

    fn write(&self, name: &str, text: &str) {
        let path = self.root.join(name);
        std::fs::create_dir_all(path.parent().expect("parent")).expect("dir");
        std::fs::write(path, text).expect("write");
    }

    fn uri(&self, name: &str) -> Url {
        Url::from_file_path(self.root.join(name)).expect("uri")
    }

    fn open(&self, name: &str, text: &str) -> Url {
        self.write(name, text);
        let uri = self.uri(name);
        self.state
            .documents
            .open(uri.clone(), text.into(), 1, "vue".into());
        uri
    }

    fn diagnostics(&self, uri: &Url) -> Value {
        let diagnostics =
            crate::runtime::block_on(DiagnosticService::collect_async(&self.state, uri));
        serde_json::to_value(diagnostics).expect("json")
    }
}

const TSCONFIG: &str = r#"{"compilerOptions":{"strict":true,"target":"ES2022","module":"ESNext","moduleResolution":"bundler","noEmit":true},"include":["src/**/*"]}"#;

fn type_error(line: u32, start: u32, end: u32, code: u32, message: &str) -> Value {
    json!([{
        "range": {
            "start": { "line": line, "character": start },
            "end": { "line": line, "character": end }
        },
        "severity": 1,
        "code": code,
        "source": "vize/types",
        "message": message
    }])
}

/// Replay a pinned LSP differential case through the shipped diagnostics path.
fn assert_lsp_case(name: &str) {
    let Some(project) = Project::new() else {
        return;
    };
    let dir = workspace()
        .join("tests/_fixtures/differential/lsp")
        .join(name);
    let read = |file: &str| std::fs::read_to_string(dir.join(file)).expect("fixture");
    let case: Value = serde_json::from_str(&read("case.json")).expect("case");
    let entry = case["entry"].as_str().expect("entry");
    let mut app = None;
    for file in case["files"].as_array().expect("files") {
        let source = read(file["source"].as_str().expect("source"));
        let path = file["runtimePath"].as_str().expect("path");
        project.write(path, &source);
        if path == entry {
            app = Some(project.open(path, &source));
        }
    }
    let app = app.expect("entry input");
    assert_eq!(
        project.diagnostics(&app),
        case["initialDiagnostics"],
        "{name}"
    );
}

#[test]
fn declared_global_slot_host_types_its_scoped_slot() {
    assert_lsp_case("declared-global-slot-host");
}

#[test]
fn declared_global_component_checks_its_props() {
    assert_lsp_case("declared-global-component-props");
}

#[test]
fn declaration_and_child_edits_reach_the_open_host() {
    let Some(project) = Project::new() else {
        return;
    };
    const DECLARATION: &str = "src/components.d.ts";
    const CHILD: &str = "src/components/GlobalBadge.vue";
    const PROVIDER: &str = "import GlobalBadge from './components/GlobalBadge.vue'\ndeclare module 'vue' { export interface GlobalComponents { GlobalBadge: typeof GlobalBadge } }\nexport {}\n";
    const NUMBER: &str = "<script setup lang=\"ts\">defineProps<{ level: number }>()</script>\n<template><span /></template>\n";
    project.write("tsconfig.json", TSCONFIG);
    project.write(CHILD, NUMBER);
    let host = project.open(
        "src/views/App.vue",
        "<template><GlobalBadge :level=\"'x'\" /></template>\n",
    );
    let invalid = type_error(
        0,
        24,
        29,
        2322,
        "Type 'string' is not assignable to type 'number'.",
    );
    assert_eq!(project.diagnostics(&host), json!([]));

    project.write(DECLARATION, PROVIDER);
    assert!(
        project
            .state
            .invalidate_global_component_references([project.uri(DECLARATION).as_str()])
    );
    assert_eq!(project.diagnostics(&host), invalid);

    // A closed child edit reaches the host through the mirrored declaration.
    project.write(CHILD, &NUMBER.replace("number", "string"));
    assert_eq!(project.diagnostics(&host), json!([]));
    project.write(CHILD, NUMBER);
    assert_eq!(project.diagnostics(&host), invalid);

    // An unsaved declaration buffer wins over its file.
    let declaration = project.uri(DECLARATION);
    project.state.documents.open(
        declaration.clone(),
        "export {}\n".into(),
        1,
        "typescript".into(),
    );
    assert_eq!(project.diagnostics(&host), json!([]));
    project.state.documents.close(&declaration);
    assert_eq!(project.diagnostics(&host), invalid);

    std::fs::remove_file(project.root.join(DECLARATION)).expect("remove");
    assert!(
        project
            .state
            .invalidate_global_component_references([declaration.as_str()])
    );
    assert_eq!(project.diagnostics(&host), json!([]));
}
