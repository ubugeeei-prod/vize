#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    reason = "independent native RPC fixture"
)]

use super::lsp_process::{LspProcess, file_uri};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

pub struct NativeOracle {
    _root: tempfile::TempDir,
    process: LspProcess,
    uri: String,
    id: i64,
    version: i64,
}

impl NativeOracle {
    pub fn new(executable: &Path, vue: &Path, config: &str, extension: &str) -> Self {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(root.path().join("src")).unwrap();
        std::fs::create_dir_all(root.path().join("node_modules")).unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink(vue, root.path().join("node_modules/vue")).unwrap();
        #[cfg(windows)]
        std::os::windows::fs::symlink_dir(vue, root.path().join("node_modules/vue")).unwrap();
        std::fs::write(root.path().join("tsconfig.json"), config).unwrap();
        let uri = file_uri(&root.path().join(format!("src/Oracle.{extension}"))).to_string();
        let mut process = LspProcess::spawn_native(root.path(), executable);
        process.send(json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{
            "processId":null,"rootUri":file_uri(root.path()),
            "capabilities":{"textDocument":{"diagnostic":{"dynamicRegistration":false,"relatedDocumentSupport":true,"relatedInformation":true}}},
            "initializationOptions":{"userPreferences":{"tsserver":{"automaticTypeAcquisition":{"enabled":false}},"inlayHints":{"variableTypes":{"enabled":true}}}}
        }}));
        let initialized = process.recv_response(1);
        assert!(initialized["result"].is_object(), "{initialized:#}");
        process.send(json!({"jsonrpc":"2.0","method":"initialized","params":{}}));
        let registration =
            process.recv_matching(|message| message["method"] == "client/registerCapability");
        assert_eq!(
            registration,
            json!({"jsonrpc":"2.0","id":"ts1","method":"client/registerCapability","params":{"registrations":[{
                "id":"typescript-config-watch-id","method":"workspace/didChangeConfiguration",
                "registerOptions":{"section":["js/ts","typescript","javascript","editor"]}
            }]}})
        );
        process.send(json!({"jsonrpc":"2.0","id":"ts1","result":null}));
        Self {
            _root: root,
            process,
            uri,
            id: 2,
            version: 0,
        }
    }

    pub fn hints(
        &mut self,
        source: &str,
        language: &str,
        range: Value,
        expected_diagnostics: Value,
    ) -> Value {
        self.version += 1;
        if self.version == 1 {
            std::fs::write(
                self._root.path().join(format!(
                    "src/Oracle.{}",
                    if language == "typescript" { "ts" } else { "js" }
                )),
                source,
            )
            .unwrap();
        }
        if let Some(root) = self.process.trace_dir() {
            std::fs::write(root.join(format!("source-{}.txt", self.version)), source).unwrap();
            std::fs::copy(
                self._root.path().join("tsconfig.json"),
                root.join("tsconfig.json"),
            )
            .unwrap();
        }
        if self.version == 1 {
            self.process.send(json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":self.uri,"languageId":language,"version":self.version,"text":source}}}));
        } else {
            self.process.send(json!({"jsonrpc":"2.0","method":"textDocument/didChange","params":{"textDocument":{"uri":self.uri,"version":self.version},"contentChanges":[{"text":source}]}}));
        }
        let diagnostics = self.request("textDocument/diagnostic", json!({}));
        assert_eq!(
            diagnostics, expected_diagnostics,
            "native whole diagnostics: {diagnostics:#}"
        );
        self.request("textDocument/inlayHint", json!({"range":range}))
    }

    fn request(&mut self, method: &str, mut params: Value) -> Value {
        params["textDocument"] = json!({"uri":self.uri});
        let id = self.id;
        self.id += 1;
        self.process
            .send(json!({"jsonrpc":"2.0","id":id,"method":method,"params":params}));
        let response = self.process.recv_response(id);
        assert!(response.get("error").is_none(), "{response:#}");
        assert_eq!(
            response,
            json!({"jsonrpc":"2.0","id":id,"result":response["result"]})
        );
        response["result"].clone()
    }

    pub fn shutdown(&mut self) {
        let id = self.id;
        self.process
            .send(json!({"jsonrpc":"2.0","id":id,"method":"shutdown"}));
        assert_eq!(
            self.process.recv_response(id),
            json!({"jsonrpc":"2.0","id":id,"result":null})
        );
        assert!(
            self.process.wait_for_transport_eof().success(),
            "native shutdown failed"
        );
    }

    pub fn project_authored_locations(&self, hints: &mut Value, authored_uri: &str) {
        for hint in hints.as_array_mut().into_iter().flatten() {
            for part in hint
                .get_mut("label")
                .and_then(Value::as_array_mut)
                .into_iter()
                .flatten()
            {
                if let Some(location) = part.get_mut("location") {
                    if location["uri"] == self.uri {
                        location["uri"] = json!(authored_uri);
                    }
                }
            }
        }
    }
}

/// Only physical file identity is normalized; every range and hint field stays.
pub fn normalize_locations(value: &mut Value) {
    match value {
        Value::Array(values) => values.iter_mut().for_each(normalize_locations),
        Value::Object(object) => {
            if let Some(Value::String(uri)) = object.get_mut("uri") {
                if let Some(path) = uri.strip_prefix("file://") {
                    if let Ok(path) = PathBuf::from(path).canonicalize() {
                        *uri = file_uri(&path).to_string();
                    }
                }
            }
            object.values_mut().for_each(normalize_locations);
        }
        _ => {}
    }
}
