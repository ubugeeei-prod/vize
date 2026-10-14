//! Standard native range hints share editor readiness and transport recovery.

use super::EditorLspSession;
use crate::lsp_client::CorsaProjectClient;
use corsa::runtime::block_on;
use serde_json::Value;
use vize_l0::{String, cstr};

struct InlayHintRequest;

impl lsp_types::request::Request for InlayHintRequest {
    type Params = Value;
    type Result = Option<Value>;
    const METHOD: &'static str = "textDocument/inlayHint";
}

impl EditorLspSession {
    fn inlay_hints(
        &mut self,
        document_uri: &str,
        start: (u32, u32),
        end: (u32, u32),
    ) -> Result<Option<Value>, String> {
        trace_native_inlay("editorReadinessInput", || {
            serde_json::json!({
                "requestUri":document_uri,
                "apiAttachment":self.configured_api.as_ref().map(|api| &api.session),
                "documentGeneration":self.document_generation,
                "readyGeneration":self.ready_generation,
                "mirroredText":self.documents.get(document_uri)
            })
        });
        let uri = self.ready_document_uri(document_uri)?;
        trace_native_inlay("editorQuery", || {
            serde_json::json!({
                "requestUri":uri,"range":{"start":start,"end":end},
                "apiAttachment":self.configured_api.as_ref().map(|api| &api.session),
                "documentGeneration":self.document_generation,
                "readyGeneration":self.ready_generation,
                "mirroredText":self.documents.get(document_uri),
                "pid":std::process::id(),"nativePidExposedBySdk":false
            })
        });
        let response = block_on(self.client.request::<InlayHintRequest>(serde_json::json!({
            "textDocument": { "uri": uri },
            "range": {
                "start": { "line": start.0, "character": start.1 },
                "end": { "line": end.0, "character": end.1 }
            }
        })))
        .map_err(|error| cstr!("Failed to request editor LSP inlay hints: {error}"));
        trace_native_inlay("editorResponse", || match &response {
            Ok(value) => serde_json::json!({"ok":true,"response":value}),
            Err(error) => serde_json::json!({"ok":false,"error":error.as_str()}),
        });
        response
    }
}

impl CorsaProjectClient {
    pub(crate) fn inlay_hints_raw(
        &mut self,
        uri: &str,
        start: (u32, u32),
        end: (u32, u32),
    ) -> Result<Option<Value>, String> {
        trace_native_inlay("projectQuery", || {
            serde_json::json!({
                "uri":uri,"start":start,"end":end,"root":self.project_root,
                "config":self.explicit_project_config,"executable":self.executable,
                "sharedEditorTransport":self.shared_editor_transport,
                "projectSessionAttachedToEditor":self.project_session_attached_to_editor,
                "sourcePresent":self.document_texts.contains_key(uri)
            })
        });
        if !self.document_texts.contains_key(uri) {
            return Ok(None);
        }
        let mut response =
            self.request_with_editor_lsp_recovery(|session| session.inlay_hints(uri, start, end))?;
        if let Some(value) = &mut response {
            // `data` and command arguments are opaque native payloads. Only
            // protocol label locations carry remappable document identities.
            for hint in value.as_array_mut().into_iter().flatten() {
                for part in hint
                    .get_mut("label")
                    .and_then(Value::as_array_mut)
                    .into_iter()
                    .flatten()
                {
                    if let Some(location) = part.get_mut("location") {
                        self.remap_result_uris(location);
                    }
                }
            }
        }
        Ok(response)
    }
}

pub(super) fn trace_native_inlay(stage: &str, value: impl FnOnce() -> Value) {
    if std::env::var_os("VIZE_TRACE_EDITOR_PREPARATION").as_deref()
        == Some(std::ffi::OsStr::new("1"))
    {
        tracing::warn!(stage, value = %value(), "native inlay hint observation");
    }
}
