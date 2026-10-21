//! Range-action diagnostics and edits share one synchronized editor snapshot.

use corsa::runtime::block_on;
use serde_json::{Value, json};
use vize_l0::{String, cstr};

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
        kinds: &[String],
    ) -> Result<Option<Value>, String> {
        if !self.document_texts.contains_key(uri) || kinds.is_empty() {
            return Ok(None);
        }
        self.request_with_editor_lsp_recovery(|session| session.code_actions(uri, range, kinds))
    }

    /// Whole-document source actions (organize, sort or remove imports, fix
    /// all) of the requested kinds. They need no diagnostics or range.
    pub(crate) fn source_actions_via_editor_lsp(
        &mut self,
        uri: &str,
        kinds: &[String],
    ) -> Result<Option<Value>, String> {
        if !self.document_texts.contains_key(uri) || kinds.is_empty() {
            return Ok(None);
        }
        self.request_with_editor_lsp_recovery(|session| session.source_actions(uri, kinds))
    }
}

impl EditorLspSession {
    fn code_actions(
        &mut self,
        uri: &str,
        range: &LspRange,
        kinds: &[String],
    ) -> Result<Option<Value>, String> {
        let requests_quick_fixes = kinds.iter().any(|kind| kind == "quickfix");
        let diagnostics = if requests_quick_fixes {
            // Client diagnostics can belong to an older edit, or to Patina.
            // Ask the same backend snapshot that computes the edits for its
            // codes. Refactors are range based and need no diagnostic.
            let report = serde_json::to_value(self.diagnostics(uri)?)
                .map_err(|error| cstr!("Failed to encode code-action diagnostics: {error}"))?;
            report
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
                .cloned()
                .collect::<Vec<_>>()
        } else {
            vec![]
        };
        if diagnostics.is_empty() && kinds.iter().all(|kind| kind == "quickfix") {
            return Ok(None);
        }
        block_on(self.client.request::<CodeActionRequest>(json!({
            "textDocument": { "uri": uri }, "range": range,
            "context": { "diagnostics": diagnostics, "only": kinds },
        })))
        .map_err(|error| cstr!("Failed to request editor LSP code actions: {error}"))
    }
}

impl EditorLspSession {
    fn source_actions(&mut self, uri: &str, kinds: &[String]) -> Result<Option<Value>, String> {
        let uri = self.ready_document_uri(uri)?;
        let empty = json!({ "line": 0, "character": 0 });
        block_on(self.client.request::<CodeActionRequest>(json!({
            "textDocument": { "uri": uri }, "range": { "start": empty, "end": empty },
            "context": { "diagnostics": [], "only": kinds },
        })))
        .map_err(|error| cstr!("Failed to request editor LSP source actions: {error}"))
    }
}

fn ranges_intersect(left: &LspRange, right: &LspRange) -> bool {
    let start = (left.start.line, left.start.character);
    let end = (left.end.line, left.end.character);
    let diagnostic_start = (right.start.line, right.start.character);
    let diagnostic_end = (right.end.line, right.end.character);
    start <= diagnostic_end && diagnostic_start <= end
}
