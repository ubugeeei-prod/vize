//! Real stdio controls with independent source/revision and disk custody.

use std::path::{Path, PathBuf};

use serde_json::{Value, json};

use super::lsp_process::{LspProcess, file_uri};
use super::{corsa_requirement, mirror, position, source_digest};

pub(super) fn required_runtime() -> Option<PathBuf> {
    let resolved = corsa_requirement::required_or_skip::<PathBuf>(None);
    if std::env::var_os("VIZE_TEST_DISABLE_TSGO").is_some() {
        return None;
    }
    // Only the explicit source-shard opt-out may skip these native stdio laws.
    // The shared helper fails closed in required CI; full/local native runs
    // also require a runtime rather than silently omitting acceptance cases.
    Some(resolved.expect("data/ARIA stdio laws require the actual native runtime"))
}

pub(super) struct Project {
    root: tempfile::TempDir,
    lsp: LspProcess,
    uri: String,
    id: i64,
    revision: u64,
    native: bool,
}

impl Project {
    pub(super) fn new(source: &str, child: &str, native: bool) -> Option<Self> {
        let runtime = required_runtime()?;
        let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap();
        let vue = workspace
            .join("playground/node_modules/vue")
            .canonicalize()
            .expect("frozen Vue required");
        let root = tempfile::Builder::new()
            .prefix("lsp-data-aria-")
            .tempdir()
            .unwrap();
        let modules = root.path().join("node_modules");
        std::fs::create_dir(&modules).unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink(&vue, modules.join("vue")).unwrap();
        #[cfg(windows)]
        std::os::windows::fs::symlink_dir(&vue, modules.join("vue")).unwrap();
        mirror::create_marker(root.path());
        std::fs::write(root.path().join("Owner.vue"), source).unwrap();
        std::fs::write(root.path().join("Declared.vue"), child).unwrap();
        std::fs::write(root.path().join("tsconfig.json"), r#"{
            "compilerOptions":{"strict":true,"target":"ES2022","module":"ESNext","moduleResolution":"bundler","noEmit":true},
            "include":["*.vue"]
        }"#).unwrap();
        std::fs::write(
            root.path().join("vize.config.json"),
            serde_json::to_vec(&json!({
                "typeChecker":{"corsaPath":runtime,"checkFallthroughAttrs":false},
                "lsp":{"lint":false,"typecheck":native,"hover":true}
            }))
            .unwrap(),
        )
        .unwrap();
        let uri = file_uri(&root.path().join("Owner.vue")).to_string();
        let mut lsp = LspProcess::spawn(root.path());
        lsp.send(
            json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{
                "processId":null,"rootUri":file_uri(root.path()),"capabilities":{},
                "initializationOptions":{"lint":false,"typecheck":native,"hover":true}
            }}),
        );
        assert!(lsp.recv_response(1)["result"]["capabilities"]["completionProvider"].is_object());
        lsp.send(json!({"jsonrpc":"2.0","method":"initialized","params":{}}));
        let mut project = Self {
            root,
            lsp,
            uri,
            id: 2,
            revision: 1,
            native,
        };
        project.lsp.send(
            json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{
                "textDocument":{"uri":project.uri,"languageId":"vue","version":1,"text":source}
            }}),
        );
        project.diagnostics(1);
        Some(project)
    }

    fn diagnostics(&mut self, version: u64) {
        self.lsp.recv_matching(|message| {
            message["method"] == "textDocument/publishDiagnostics"
                && message["params"]["uri"] == self.uri
                && message["params"]["version"] == version
        });
    }

    pub(super) fn change(&mut self, source: &str) {
        // This process opens exactly one document. Every full change owns the
        // next monotonic source revision, independent of a completion result.
        self.revision += 1;
        self.lsp.send(json!({"jsonrpc":"2.0","method":"textDocument/didChange","params":{
            "textDocument":{"uri":self.uri,"version":self.revision},"contentChanges":[{"text":source}]
        }}));
        self.diagnostics(self.revision);
    }

    pub(super) fn assert_response(&mut self, source: &str, needle: &str, expected: Value) -> Value {
        let id = self.id;
        self.id += 1;
        self.lsp.send(json!({"jsonrpc":"2.0","id":id,"method":"textDocument/completion","params":{
            "textDocument":{"uri":self.uri},"position":position(source,needle),"context":{"triggerKind":1}
        }}));
        let response = self.lsp.recv_response(id);
        assert_eq!(response, json!({"jsonrpc":"2.0","id":id,"result":expected}));
        response["result"].clone()
    }

    pub(super) fn declared_data(&self, source: &str, name: &str) -> Option<Value> {
        if !self.native {
            return None;
        }
        let observed = mirror::observe(self.root.path(), source, self.lsp.trace_dir());
        let mut data: Value = serde_json::from_str(include_str!(
            "../../../../tests/_fixtures/differential/lsp/data-aria-attributes-8015/declared-native.expected.json"
        )).unwrap();
        let payload = &mut data["vizeCompletion"];
        for key in ["label", "uri", "requestUri", "revision"] {
            assert!(payload[key].is_null());
        }
        for key in ["name", "fileName", "position"] {
            assert!(payload["item"]["data"][key].is_null());
        }
        for key in ["label", "filterText", "insertText"] {
            assert!(payload["item"][key].is_null());
        }
        payload["label"] = json!(name);
        payload["uri"] = json!(self.uri);
        payload["revision"] = json!(self.revision);
        payload["requestUri"] = json!(file_uri(&observed.path));
        // Both authored Declared.vue properties are optional. TypeScript's
        // member carrier decorates only the backend label with `?`, retaining
        // the undecorated name for filtering, insertion and lazy resolve data.
        payload["item"]["label"] = json!(format!("{name}?"));
        payload["item"]["filterText"] = json!(name);
        payload["item"]["insertText"] = json!(name);
        payload["item"]["data"]["name"] = json!(name);
        payload["item"]["data"]["fileName"] = json!(observed.path);
        payload["item"]["data"]["position"] = json!(observed.position);
        Some(data)
    }

    pub(super) fn assert_disk(&self, source: &str, child: &str) {
        for (name, expected) in [("Owner.vue", source), ("Declared.vue", child)] {
            let actual = std::fs::read_to_string(self.root.path().join(name)).unwrap();
            assert_eq!(actual, expected);
            assert_eq!(source_digest(&actual), source_digest(expected));
        }
    }

    pub(super) fn shutdown(&mut self) {
        self.lsp
            .send(json!({"jsonrpc":"2.0","id":self.id,"method":"shutdown"}));
        assert_eq!(
            self.lsp.recv_response(self.id),
            json!({"jsonrpc":"2.0","id":self.id,"result":null})
        );
        self.lsp.send(json!({"jsonrpc":"2.0","method":"exit"}));
        assert!(self.lsp.wait_for_exit().success());
    }
}
