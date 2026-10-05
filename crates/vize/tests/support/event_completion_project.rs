//! Source-built stdio project with complete messages retained before assertions.
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    reason = "wire fixtures use std strings and protocol JSON"
)]

use std::path::Path;

use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use vize_carton::corsa_resolver::discover_corsa_in_ancestors;

use super::lsp_process::{LspProcess, file_uri};

pub struct Project {
    root: tempfile::TempDir,
    pub app_uri: String,
    pub child_uri: String,
    lsp: LspProcess,
    next_id: i64,
    capture: std::path::PathBuf,
    receipt: Value,
}

impl Project {
    pub fn new(name: &str, app: &str, child: &str) -> Self {
        let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap();
        let runtime = discover_corsa_in_ancestors(workspace)
            .expect("event completion tests require the actual TypeScript native runtime");
        let vue = workspace
            .join("playground/node_modules/vue")
            .canonicalize()
            .expect("event completion tests require frozen Playground Vue");
        let root = tempfile::Builder::new()
            .prefix("lsp-events-")
            .tempdir()
            .unwrap();
        let modules = root.path().join("node_modules");
        std::fs::create_dir_all(&modules).unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink(&vue, modules.join("vue")).unwrap();
        #[cfg(windows)]
        std::os::windows::fs::symlink_dir(&vue, modules.join("vue")).unwrap();
        std::fs::create_dir_all(root.path().join("src")).unwrap();
        std::fs::write(root.path().join("src/App.vue"), app).unwrap();
        std::fs::write(root.path().join("src/MySwitch.vue"), child).unwrap();
        let tsconfig = include_str!(
            "../../../../tests/_fixtures/differential/lsp/component-native-events/tsconfig.json"
        );
        std::fs::write(root.path().join("tsconfig.json"), tsconfig).unwrap();
        let original_config = include_str!(
            "../../../../tests/_fixtures/differential/lsp/component-native-events/vize.config.json.txt"
        );
        let mut config: Value = serde_json::from_str(original_config).unwrap();
        config["typeChecker"] = json!({ "corsaPath": runtime, "checkFallthroughAttrs": false });
        std::fs::write(
            root.path().join("vize.config.json"),
            serde_json::to_vec(&config).unwrap(),
        )
        .unwrap();
        let capture = workspace
            .join("target/nextest/full/lsp-event-completions")
            .join(name)
            .with_extension("json");
        std::fs::create_dir_all(capture.parent().unwrap()).unwrap();
        let archive = workspace.join(".artifacts/rust-test-archive/receipt.json");
        let receipt = json!({
            "sourceHead": std::env::var("GITHUB_SHA").ok(),
            "binary": { "path": env!("CARGO_BIN_EXE_vize"), "sha256": hash(&std::fs::read(env!("CARGO_BIN_EXE_vize")).unwrap()) },
            "nativeRuntime": { "path": runtime, "sha256": hash(&std::fs::read(&runtime).unwrap()) }, "vue": vue,
            "archive": archive.exists().then(|| serde_json::from_slice::<Value>(&std::fs::read(archive).unwrap()).unwrap()),
            "inputs": { "app": app, "child": child, "tsconfig": tsconfig, "originalConfig": original_config, "runtimeConfig": config },
            "messages": []
        });
        let app_uri = file_uri(&root.path().join("src/App.vue")).to_string();
        let child_uri = file_uri(&root.path().join("src/MySwitch.vue")).to_string();
        let lsp = LspProcess::spawn(root.path());
        let mut project = Self {
            root,
            app_uri,
            child_uri,
            lsp,
            next_id: 2,
            capture,
            receipt,
        };
        project.send(
            json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {
                "processId": null, "rootUri": file_uri(project.root.path()), "capabilities": {},
                "initializationOptions": { "lint": false, "typecheck": true, "hover": true }
            }}),
        );
        let initialized = project.receive(|message| message["id"] == 1);
        assert!(
            initialized["result"]["capabilities"]["completionProvider"].is_object(),
            "{initialized}"
        );
        project.send(json!({ "jsonrpc": "2.0", "method": "initialized", "params": {} }));
        project.open(&project.child_uri.clone(), child);
        project.open(&project.app_uri.clone(), app);
        project
    }

    fn send(&mut self, message: Value) {
        self.record("send", message.clone());
        self.lsp.send(message);
    }

    fn record(&mut self, direction: &str, message: Value) {
        self.receipt["messages"]
            .as_array_mut()
            .unwrap()
            .push(json!({ "direction": direction, "message": message }));
        std::fs::write(
            &self.capture,
            serde_json::to_vec_pretty(&self.receipt).unwrap(),
        )
        .unwrap();
    }

    fn receive(&mut self, mut matches: impl FnMut(&Value) -> bool) -> Value {
        let receipt = &mut self.receipt;
        let capture = &self.capture;
        self.lsp.recv_matching(|message| {
            receipt["messages"].as_array_mut().unwrap().push(json!({
                "direction": "receive", "message": message
            }));
            std::fs::write(capture, serde_json::to_vec_pretty(receipt).unwrap()).unwrap();
            matches(message)
        })
    }

    fn open(&mut self, uri: &str, source: &str) {
        self.send(
            json!({ "jsonrpc": "2.0", "method": "textDocument/didOpen", "params": {
                "textDocument": { "uri": uri, "languageId": "vue", "version": 1, "text": source }
            }}),
        );
        self.diagnostics(uri, 1);
    }

    pub fn change(&mut self, child: bool, source: &str, version: i64) {
        let uri = if child {
            &self.child_uri
        } else {
            &self.app_uri
        }
        .clone();
        self.send(json!({ "jsonrpc": "2.0", "method": "textDocument/didChange", "params": {
            "textDocument": { "uri": uri, "version": version }, "contentChanges": [{ "text": source }]
        }}));
        self.diagnostics(&uri, version);
    }

    fn diagnostics(&mut self, uri: &str, version: i64) {
        self.receive(|message| {
            message["method"] == "textDocument/publishDiagnostics"
                && message["params"]["uri"] == uri
                && message["params"]["version"] == version
        });
    }

    pub fn assert_response(&mut self, method: &str, position: Value, expected: Value) {
        let id = self.next_id;
        self.next_id += 1;
        let mut params = json!({ "textDocument": { "uri": self.app_uri }, "position": position });
        if method == "textDocument/completion" {
            params["context"] = json!({ "triggerKind": 1 });
        }
        self.send(json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params }));
        let response = self.receive(|message| message["id"] == id);
        assert_eq!(
            response,
            json!({ "jsonrpc": "2.0", "id": id, "result": expected }),
            "{}",
            self.capture.display()
        );
    }

    pub fn assert_disk_child(&self, expected: &str) {
        assert_eq!(
            std::fs::read_to_string(self.root.path().join("src/MySwitch.vue")).unwrap(),
            expected
        );
    }

    pub fn shutdown(&mut self) {
        let id = self.next_id;
        self.send(json!({ "jsonrpc": "2.0", "id": id, "method": "shutdown" }));
        let response = self.receive(|message| message["id"] == id);
        assert_eq!(
            response,
            json!({ "jsonrpc": "2.0", "id": id, "result": null })
        );
        self.send(json!({ "jsonrpc": "2.0", "method": "exit" }));
        assert!(self.lsp.wait_for_exit().success());
        self.receipt["successfulShutdown"] = json!(true);
        self.record("receipt", json!({ "complete": true }));
    }
}

fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
