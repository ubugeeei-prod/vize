// Each integration target uses a different subset of this shared fixture.
#![allow(dead_code)]

use super::lsp_process::{LspProcess, file_uri};
use serde_json::{Value, json};
use std::path::Path;
use vize_s0::corsa_resolver::discover_corsa_in_ancestors;

pub struct Fixture {
    _project: tempfile::TempDir,
    pub uri: String,
    lsp: LspProcess,
    next_id: i64,
}

impl Fixture {
    pub fn new(source: &str, enabled: bool) -> Self {
        Self::new_with_cross_file(source, enabled, false)
    }

    pub fn new_with_cross_file(source: &str, enabled: bool, cross_file: bool) -> Self {
        Self::new_with_options(source, enabled, cross_file, false)
    }

    pub fn new_with_vue(source: &str) -> Self {
        Self::new_with_options(source, false, false, true)
    }

    pub fn new_with_vue_and_cross_file(source: &str) -> Self {
        Self::new_with_options(source, false, true, true)
    }

    pub fn new_with_vue_and_patterns(source: &str) -> Self {
        Self::new_with_options(source, true, false, true)
    }

    fn new_with_options(source: &str, enabled: bool, cross_file: bool, real_vue: bool) -> Self {
        let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap();
        let runtime = discover_corsa_in_ancestors(workspace)
            .expect("Vue editor tests require the workspace TypeScript native runtime");
        let cases = workspace.join("target/vize-tests/tests");
        std::fs::create_dir_all(&cases).unwrap();
        let project = tempfile::Builder::new()
            .prefix("lsp-vue-")
            .tempdir_in(cases)
            .unwrap();
        if real_vue {
            let vue = workspace
                .join("playground/node_modules/vue")
                .canonicalize()
                .expect("Vue editor tests require the frozen Playground Vue dependency");
            let modules = project.path().join("node_modules");
            std::fs::create_dir_all(&modules).unwrap();
            #[cfg(unix)]
            std::os::unix::fs::symlink(&vue, modules.join("vue")).unwrap();
            #[cfg(windows)]
            std::os::windows::fs::symlink_dir(&vue, modules.join("vue")).unwrap();
        }
        std::fs::write(project.path().join("tsconfig.json"), r#"{
            "compilerOptions": { "strict": true, "target": "ES2022", "module": "ESNext", "moduleResolution": "bundler", "noEmit": true },
            "include": ["*.vue"]
        }"#).unwrap();
        std::fs::write(
            project.path().join("vize.config.json"),
            serde_json::to_vec(&json!({
                "experimentals": { "patternedTemplate": enabled },
                "typeChecker": { "corsaPath": runtime, "checkFallthroughAttrs": false },
                "lsp": { "lint": false, "typecheck": true, "hover": true, "crossFile": cross_file }
            }))
            .unwrap(),
        )
        .unwrap();
        let path = project.path().join("App.vue");
        std::fs::write(&path, source).unwrap();
        let uri = file_uri(&path).to_string();
        let mut lsp = LspProcess::spawn(project.path());
        lsp.send(
            json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {
                "processId": null, "rootUri": file_uri(project.path()), "capabilities": {},
                "initializationOptions": { "lint": false, "typecheck": true, "hover": true, "crossFile": cross_file }
            }}),
        );
        assert!(lsp.recv_response(1)["result"].is_object());
        lsp.send(json!({ "jsonrpc": "2.0", "method": "initialized", "params": {} }));
        Self {
            _project: project,
            uri,
            lsp,
            next_id: 2,
        }
    }

    pub fn open(&mut self, source: &str) -> Value {
        self.lsp.send(json!({ "jsonrpc": "2.0", "method": "textDocument/didOpen", "params": {
            "textDocument": { "uri": self.uri, "languageId": "vue", "version": 1, "text": source }
        }}));
        self.diagnostics(1)
    }

    pub fn write_file(&self, name: &str, source: &str) -> String {
        let path = self._project.path().join(name);
        std::fs::write(&path, source).unwrap();
        file_uri(&path).to_string()
    }

    pub fn change(&mut self, source: &str, version: i64) -> Value {
        self.lsp.send(
            json!({ "jsonrpc": "2.0", "method": "textDocument/didChange", "params": {
                "textDocument": { "uri": self.uri, "version": version },
                "contentChanges": [{ "text": source }]
            }}),
        );
        self.diagnostics(version)
    }

    fn diagnostics(&mut self, version: i64) -> Value {
        self.lsp.recv_matching(|message| {
            message["method"] == "textDocument/publishDiagnostics"
                && message["params"]["uri"] == self.uri
                && message["params"]["version"] == version
        })["params"]["diagnostics"]
            .clone()
    }

    pub fn request(&mut self, method: &str, source: &str, needle: &str) -> Value {
        self.request_with(method, source, needle, json!({}))
    }

    pub fn request_with(
        &mut self,
        method: &str,
        source: &str,
        needle: &str,
        mut params: Value,
    ) -> Value {
        let id = self.next_id;
        self.next_id += 1;
        params["textDocument"] = json!({ "uri": self.uri });
        params["position"] = position(source, needle);
        self.lsp
            .send(json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params }));
        let response = self.lsp.recv_response(id);
        assert!(response.get("error").is_none(), "{response:#}");
        response["result"].clone()
    }

    pub fn shutdown(&mut self) {
        let id = self.next_id;
        self.lsp
            .send(json!({ "jsonrpc": "2.0", "id": id, "method": "shutdown" }));
        assert!(self.lsp.recv_response(id)["result"].is_null());
        self.lsp.send(json!({ "jsonrpc": "2.0", "method": "exit" }));
        assert!(self.lsp.wait_for_exit().success());
    }
}

pub fn position(source: &str, needle: &str) -> Value {
    let prefix = &source[..source.find(needle).expect("fixture needle")];
    let prefix = prefix.replace("\r\n", "\n").replace('\r', "\n");
    let line = prefix.bytes().filter(|&b| b == b'\n').count();
    let start = prefix.rfind('\n').map_or(0, |n| n + 1);
    json!({ "line": line, "character": prefix[start..].encode_utf16().count() })
}
