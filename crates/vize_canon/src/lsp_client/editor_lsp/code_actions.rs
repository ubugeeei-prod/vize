//! Quick-fix diagnostics and edits share one synchronized editor snapshot.

use corsa::runtime::block_on;
use serde_json::{Value, json};
use vize_s0::{String, cstr};

use super::{CorsaProjectClient, EditorLspSession};
use crate::LspRange;

struct CodeActionRequest;

impl lsp_types::request::Request for CodeActionRequest {
    type Params = Value;
    type Result = Option<Value>;
    const METHOD: &'static str = "textDocument/codeAction";
}

impl CorsaProjectClient {
    pub(crate) fn code_actions_via_editor_lsp(
        &mut self,
        uri: &str,
        range: &LspRange,
    ) -> Result<Option<Value>, String> {
        if !self.document_texts.contains_key(uri) {
            return Ok(None);
        }
        self.request_with_editor_lsp_recovery(|session| session.code_actions(uri, range))
    }
}

impl EditorLspSession {
    fn code_actions(&mut self, uri: &str, range: &LspRange) -> Result<Option<Value>, String> {
        // Client diagnostics can belong to an older edit, or to Patina. Ask
        // the same backend snapshot that will compute the edits for its codes.
        let report = serde_json::to_value(self.diagnostics(uri)?)
            .map_err(|error| cstr!("Failed to encode code-action diagnostics: {error}"))?;
        let diagnostics: Vec<_> = report
            .get("items")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter(|diagnostic| {
                diagnostic
                    .get("range")
                    .and_then(|range| serde_json::from_value::<LspRange>(range.clone()).ok())
                    .is_some_and(|diagnostic| ranges_intersect(range, &diagnostic))
            })
            .collect();
        if diagnostics.is_empty() {
            return Ok(None);
        }
        block_on(self.client.request::<CodeActionRequest>(json!({
            "textDocument": { "uri": uri }, "range": range,
            "context": { "diagnostics": diagnostics, "only": ["quickfix"] },
        })))
        .map_err(|error| cstr!("Failed to request editor LSP code actions: {error}"))
    }
}

fn ranges_intersect(left: &LspRange, right: &LspRange) -> bool {
    let start = (left.start.line, left.start.character);
    let end = (left.end.line, left.end.character);
    let diagnostic_start = (right.start.line, right.start.character);
    let diagnostic_end = (right.end.line, right.end.character);
    start <= diagnostic_end && diagnostic_start <= end
}
