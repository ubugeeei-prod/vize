//! Literal original configs and full RPC envelopes, using the existing process.
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    reason = "authored full protocol fixtures"
)]

use super::lsp_process::{LspProcess, file_uri};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use vize_carton::corsa_resolver::discover_corsa_in_ancestors;

pub struct Project {
    root: tempfile::TempDir,
    process: LspProcess,
    pub uri: String,
    id: i64,
}

pub fn workspace() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
}

fn vue() -> PathBuf {
    std::fs::read_dir(workspace().join("node_modules/.pnpm"))
        .unwrap()
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .starts_with("vue@3.5.41")
        })
        .map(|entry| entry.path().join("node_modules/vue"))
        .find(|path| path.join("package.json").is_file())
        .expect("the existing fixture-only pin must supply Vue 3.5.41")
        .canonicalize()
        .unwrap()
}

impl Project {
    pub fn new(source: &str, typecheck: bool) -> Self {
        discover_corsa_in_ancestors(workspace()).expect("real native provider");
        let cases = workspace().join("target/vize-tests/tests");
        std::fs::create_dir_all(&cases).unwrap();
        let root = tempfile::Builder::new()
            .prefix("lsp-component-tags-")
            .tempdir_in(cases)
            .unwrap();
        std::fs::create_dir(root.path().join("src")).unwrap();
        std::fs::create_dir(root.path().join("node_modules")).unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink(vue(), root.path().join("node_modules/vue")).unwrap();
        #[cfg(windows)]
        std::os::windows::fs::symlink_dir(vue(), root.path().join("node_modules/vue")).unwrap();
        for (name, bytes) in [
            (
                "tsconfig.json",
                include_str!(
                    "../../../../tests/_fixtures/differential/lsp/component-tag-classification/tsconfig.json.txt"
                ),
            ),
            (
                "vize.config.json",
                include_str!(
                    "../../../../tests/_fixtures/differential/lsp/component-tag-classification/vize.config.json.txt"
                ),
            ),
            (
                "src/MyButton.vue",
                include_str!(
                    "../../../../tests/_fixtures/differential/lsp/component-tag-classification/MyButton.vue.txt"
                ),
            ),
            ("src/constants.ts", "export const ImportedCount = 1;\n"),
            ("src/App.vue", source),
        ] {
            std::fs::write(root.path().join(name), bytes).unwrap();
            assert_eq!(
                std::fs::read(root.path().join(name)).unwrap(),
                bytes.as_bytes()
            );
        }
        let uri = file_uri(&root.path().join("src/App.vue")).to_string();
        let mut process = LspProcess::spawn(root.path());
        process.send(
            json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{
                "processId":null,"rootUri":file_uri(root.path()),"capabilities":{},
                "initializationOptions":{"lint":false,"typecheck":typecheck,"editor":true}
            }}),
        );
        assert!(process.recv_response(1)["result"].is_object());
        process.send(json!({"jsonrpc":"2.0","method":"initialized","params":{}}));
        Self {
            root,
            process,
            uri,
            id: 2,
        }
    }

    pub fn open(&mut self, source: &str) {
        self.process.send(
            json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{
                "textDocument":{"uri":self.uri,"languageId":"vue","version":1,"text":source}
            }}),
        );
        self.diagnostics(1);
    }

    pub fn change(&mut self, source: &str, version: i64) {
        self.process.send(
            json!({"jsonrpc":"2.0","method":"textDocument/didChange","params":{
                "textDocument":{"uri":self.uri,"version":version},"contentChanges":[{"text":source}]
            }}),
        );
        self.diagnostics(version);
    }

    fn diagnostics(&mut self, version: i64) {
        self.process.recv_matching(|value| {
            value["method"] == "textDocument/publishDiagnostics"
                && value["params"]["uri"] == self.uri
                && value["params"]["version"] == version
        });
    }

    pub fn completion(&mut self, source: &str, caret: usize, expected: Value) {
        let before = &source[..caret];
        let line = before.bytes().filter(|byte| *byte == b'\n').count();
        let character = before.rsplit('\n').next().unwrap().encode_utf16().count();
        let id = self.id;
        self.id += 1;
        self.process.send(json!({"jsonrpc":"2.0","id":id,"method":"textDocument/completion","params":{
            "textDocument":{"uri":self.uri},"position":{"line":line,"character":character},"context":{"triggerKind":1}
        }}));
        assert_eq!(
            self.process.recv_response(id),
            json!({"jsonrpc":"2.0","id":id,"result":expected})
        );
    }

    pub fn unchanged_disk(&self, original: &str) {
        assert_eq!(
            std::fs::read_to_string(self.root.path().join("src/App.vue")).unwrap(),
            original
        );
    }

    pub fn shutdown(&mut self) {
        self.process
            .send(json!({"jsonrpc":"2.0","id":self.id,"method":"shutdown"}));
        assert_eq!(
            self.process.recv_response(self.id),
            json!({"jsonrpc":"2.0","id":self.id,"result":null})
        );
        self.process.send(json!({"jsonrpc":"2.0","method":"exit"}));
        assert!(self.process.wait_for_exit().success());
    }
}
