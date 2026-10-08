//! Original project/configuration and existing editor assembly trace boundary.
#![expect(
    clippy::disallowed_types,
    clippy::expect_used,
    reason = "complete test custody uses std strings and fails closed by panicking"
)]

use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use tower_lsp::lsp_types::{Position, TextDocumentContentChangeEvent, Url};
use vize_carton::corsa_resolver::discover_corsa_in_ancestors;

use super::super::{DiagnosticService, assembly_parity_custody};
use crate::server::ServerState;

pub(super) const TOGGLE: &str = include_str!(
    "../../../../../../tests/_fixtures/differential/lsp/rename-library-refusal/8010/Toggle.vue.txt"
);
pub(super) const APP: &str = include_str!(
    "../../../../../../tests/_fixtures/differential/lsp/rename-library-refusal/8010/App.vue.txt"
);
pub(super) const MISSING_CALL: &str = include_str!(
    "../../../../../../tests/_fixtures/differential/lsp/rename-library-refusal/8010/supplemental/partial-application/MissingCallToggle.vue.txt"
);
pub(super) const UPDATED_TOGGLE: &str = include_str!(
    "../../../../../../tests/_fixtures/differential/lsp/event-rename/8010/UpdatedToggle.vue.txt"
);
pub(super) const UPDATED_APP: &str = include_str!(
    "../../../../../../tests/_fixtures/differential/lsp/event-rename/8010/UpdatedApp.vue.txt"
);
const CONFIG: &str = r#"{
  "compilerOptions": {
    "lib": ["ESNext", "DOM"], "strict": true,
    "moduleResolution": "Bundler", "module": "ESNext", "target": "ESNext"
  },
  "include": ["src/**/*.vue"]
}"#;
const INVALID: &str = "<script setup lang=\"ts\">\nconst nativeGuard: number = 'wrong';\n</script>\n<template>{{ nativeGuard }}</template>\n";
const VALID: &str = "<script setup lang=\"ts\">\nconst nativeGuard: number = 1;\n</script>\n<template>{{ nativeGuard }}</template>\n";

pub(super) struct Fixture {
    project: tempfile::TempDir,
    pub(super) state: ServerState,
    runtime: PathBuf,
    inputs: Value,
    trace_stages: std::cell::RefCell<Vec<Vec<String>>>,
}

impl Fixture {
    pub(super) fn new(toggle: &str, app: &str) -> Self {
        let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("crates")
            .parent()
            .expect("workspace");
        // This mandatory witness never inherits the optional unit-helper optout.
        let runtime =
            discover_corsa_in_ancestors(workspace).expect("required actual native runtime");
        let cases = workspace.join("target/vize-tests/tests");
        std::fs::create_dir_all(&cases).expect("cases");
        let project = tempfile::Builder::new()
            .prefix("editor-event-partial-")
            .tempdir_in(cases)
            .expect("original project");
        let root = project.path();
        std::fs::create_dir_all(root.join("src")).expect("src");
        std::fs::create_dir_all(root.join("node_modules")).expect("node modules");
        let vue = workspace
            .join("playground/node_modules/vue")
            .canonicalize()
            .expect("actual frozen Vue");
        #[cfg(unix)]
        std::os::unix::fs::symlink(&vue, root.join("node_modules/vue")).expect("actual Vue link");
        #[cfg(windows)]
        std::os::windows::fs::symlink_dir(&vue, root.join("node_modules/vue"))
            .expect("actual Vue link");
        std::fs::write(root.join("tsconfig.json"), CONFIG).expect("original tsconfig");
        let flags = json!({"lint":true,"typecheck":true,"hover":true,"crossFile":true});
        let config = json!({"typeChecker":{"corsaPath":runtime},"lsp":flags});
        std::fs::write(
            root.join("vize.config.json"),
            serde_json::to_vec(&config).expect("config JSON"),
        )
        .expect("config");
        for (name, source) in [("Toggle.vue", toggle), ("App.vue", app)] {
            std::fs::write(root.join("src").join(name), source).expect("original authored file");
        }
        let state = ServerState::new();
        state.load_workspace_config(root);
        state.apply_initialize_workspace_folders(None, Some(root));
        state.apply_lsp_initialization_options(Some(&flags));
        state.set_workspace_root(root.to_path_buf());
        let inputs = json!({"toggle":toggle,"app":app,"tsconfig":CONFIG,"vizeConfig":config,"initializationOptions":flags,
            "vuePath":vue,"vuePackage":std::fs::read_to_string(vue.join("package.json")).expect("actual Vue package"),
            "vueDeclarations":std::fs::read_to_string(vue.join("dist/vue.d.ts")).expect("actual Vue declarations"),
            "nativeGuardDisk":INVALID,"nativeGuardVersion2Overlay":VALID});
        Self {
            project,
            state,
            runtime,
            inputs,
            trace_stages: Default::default(),
        }
    }

    pub(super) fn uri(&self, name: &str) -> Url {
        Url::from_file_path(self.project.path().join("src").join(name)).expect("authored URI")
    }

    pub(super) fn open(&self, name: &str, text: &str) {
        let uri = self.uri(name);
        self.state
            .documents
            .open(uri.clone(), text.to_owned(), 1, "vue".to_owned());
        self.state.update_virtual_docs(&uri, text);
    }

    pub(super) fn change(&self, name: &str, text: &str, version: i32) {
        let uri = self.uri(name);
        assert!(self.state.documents.apply_changes(
            &uri,
            vec![TextDocumentContentChangeEvent {
                range: None,
                range_length: None,
                text: text.to_owned()
            }],
            version
        ));
        self.state.update_virtual_docs(&uri, text);
    }

    pub(super) fn write(&self, name: &str, text: &str) {
        std::fs::write(self.project.path().join("src").join(name), text)
            .expect("actual disk application");
    }

    pub(super) fn collect(&self, name: &str, rows: &mut Vec<Value>) -> Value {
        let uri = self.uri(name);
        let observation = assembly_parity_custody::observe(&self.state, &uri);
        let diagnostics = crate::runtime::block_on(async {
            let _scope = self.state.corsa_request_scope().await;
            DiagnosticService::collect_async(&self.state, &uri).await
        });
        let custody = assembly_parity_custody::take();
        drop(observation);
        self.trace_stages.borrow_mut().push(
            custody
                .iter()
                .map(|row| row["stage"].as_str().expect("trace stage").to_owned())
                .collect(),
        );
        rows.push(json!({"file":name,"uri":uri,"version":self.state.documents.version(&uri),
            "disk":std::fs::read_to_string(self.project.path().join("src").join(name)).expect("actual disk"),
            "custody":custody,"finalDiagnostics":diagnostics}));
        serde_json::to_value(diagnostics).expect("complete diagnostics")
    }

    pub(super) fn prove_native(&self, rows: &mut Vec<Value>) -> Value {
        self.write("NativeGuard.vue", INVALID);
        self.open("NativeGuard.vue", INVALID);
        let invalid = self.collect("NativeGuard.vue", rows);
        // The original stdio guard repairs only its editor overlay, not disk.
        self.change("NativeGuard.vue", VALID, 2);
        let valid = self.collect("NativeGuard.vue", rows);
        json!([invalid, valid])
    }

    pub(super) fn file(&self, name: &str, text: &str, version: i32, diagnostics: Value) -> Value {
        json!({"file":name,"text":text,"version":version,"diagnostics":diagnostics,
            "disk":std::fs::read_to_string(self.project.path().join("src").join(name)).expect("actual applied disk")})
    }

    pub(super) fn capture(
        &self,
        newline: &str,
        expected: &Value,
        actual: &Value,
        custody: &[Value],
    ) {
        let packet = json!({"context":"original partial event editor assembly witness","newline":newline,
            "sourceSha":std::env::var("SOURCE_SHA").ok(),"requireTsgo":std::env::var("VIZE_TEST_REQUIRE_TSGO").ok(),
            "disableTsgo":std::env::var("VIZE_TEST_DISABLE_TSGO").ok(),"runtime":self.runtime,"inputs":self.inputs,
            "expected":expected,"actual":actual,"custody":custody});
        println!("complete editor partial diagnostic witness: {packet}");
        let root = std::env::var_os("VIZE_TEST_FIX_HISTORY_CAPTURE_DIR")
            .map(PathBuf::from)
            .or_else(|| {
                std::env::var_os("NEXTEST_PROFILE").map(|profile| {
                    Path::new(env!("CARGO_MANIFEST_DIR"))
                        .parent()
                        .expect("crates")
                        .parent()
                        .expect("workspace")
                        .join("target/nextest")
                        .join(profile)
                })
            });
        if let Some(root) = root {
            let root = root.join("editor-event-partial-diagnostics");
            std::fs::create_dir_all(&root).expect("capture dir");
            std::fs::write(
                root.join(self.project.path().file_name().expect("unique project"))
                    .with_extension("json"),
                serde_json::to_vec_pretty(&packet).expect("whole packet JSON"),
            )
            .expect("whole custody capture");
        }
        let expected_stages: Vec<_> = [
            "prepared",
            "opened",
            "native_query_started",
            "native_query_returned",
            "native_complete",
        ]
        .map(str::to_owned)
        .into_iter()
        .collect();
        assert_eq!(
            *self.trace_stages.borrow(),
            vec![expected_stages; 8],
            "all native bodies completed with full trace"
        );
    }
}

pub(super) fn offset(source: &str, position: Position) -> usize {
    vize_l0::line_index::LineBreaks::Lsp
        .position_to_offset(source, position.line, position.character)
        .expect("complete authored UTF16 position")
}
