use std::sync::Mutex;

use serde_json::{Value, json};

/// Answers the server-to-client requests the exact tsgo conformance tests rely on, and records the
/// dynamic registrations pushed through `client/registerCapability`. Capturing them is what lets a
/// test assert that the server actually registers content mapper document synchronization for the
/// authored extension: a responder that blindly answers `null` would pass even if the request never
/// arrived.
#[derive(Default)]
pub struct EditorResponder(Mutex<Vec<Value>>);

impl EditorResponder {
    pub fn respond_to(&self, method: &str, params: &Value) -> Value {
        match method {
            "workspace/configuration" => {
                let count = params["items"].as_array().map_or(0, Vec::len);
                Value::Array(vec![Value::Null; count])
            }
            "client/registerCapability" => {
                let registrations = params["registrations"].as_array();
                self.0
                    .lock()
                    .unwrap()
                    .extend(registrations.into_iter().flatten().cloned());
                Value::Null
            }
            _ => Value::Null,
        }
    }

    /// Asserts the server registered `textDocument/didOpen` for `**/*.vue`, which is how a `.vue`
    /// file starts flowing to the server once a content mapper claims it.
    pub fn assert_vue_did_open_registration(&self) {
        let recorded = self.0.lock().unwrap();
        let did_open = recorded
            .iter()
            .find(|registration| registration["id"] == json!("content-mapper-did-open"))
            .unwrap_or_else(|| panic!("missing didOpen registration: {recorded:#?}"));
        assert_eq!(
            did_open["method"],
            json!("textDocument/didOpen"),
            "{did_open:#}"
        );
        let selector = did_open["registerOptions"]["documentSelector"]
            .as_array()
            .unwrap_or_else(|| panic!("missing didOpen document selector: {did_open:#}"));
        assert!(
            selector
                .iter()
                .any(|filter| filter["pattern"] == json!("**/*.vue")),
            "{did_open:#}"
        );
    }
}
