use super::{
    CorsaProjectClient, DiagnosticFetch, LspDiagnostic,
    diagnostics_api::{document_identifier_uri, flatten_file_diagnostics, map_project_diagnostics},
    session::uri_document_identifier,
    utils::convert_diagnostics,
};
use crate::file_uri::file_uri_to_path;
use corsa::{CorsaError, runtime::block_on};
use lsp_types::{Diagnostic, DocumentDiagnosticReport, DocumentDiagnosticReportResult};
use std::time::Duration;
use vize_s0::{FxHashMap, String, cstr};

type DiagnosticBatch = Vec<(String, Vec<LspDiagnostic>)>;
type EditorLspDiagnosticDocuments = FxHashMap<String, String>;
type EditorLspDiagnosticPairs = Vec<(String, String)>;
const LSP_DIAGNOSTICS_BATCH_CHUNK_SIZE: usize = 128;
const LSP_DIAGNOSTICS_BATCH_TRANSIENT_RETRIES: usize = 1;

impl CorsaProjectClient {
    /// Get cached diagnostics for a URI.
    pub fn get_diagnostics(&self, uri: &str) -> Vec<LspDiagnostic> {
        self.diagnostics
            .get(uri)
            .map(|diagnostics| convert_diagnostics(diagnostics))
            .unwrap_or_default()
    }

    /// Request diagnostics for a single document.
    pub fn request_diagnostics(&mut self, uri: &str) -> Result<Vec<LspDiagnostic>, String> {
        self.request_diagnostics_full(uri)
            .map(|fetch| convert_diagnostics(&fetch.diagnostics))
    }

    /// Request diagnostics for multiple URIs in batch.
    pub fn request_diagnostics_batch(
        &mut self,
        uris: &[String],
    ) -> Result<Vec<(String, Vec<LspDiagnostic>)>, String> {
        if self.has_materialized_documents(uris)
            && let Some(results) = self.request_diagnostics_batch_via_materialized_files(uris)?
        {
            return Ok(results);
        }

        if self.can_batch_with_project_diagnostics(uris)
            && let Some(results) = self.request_diagnostics_batch_via_project_api(uris)?
        {
            return Ok(results);
        }

        if let Some(results) = self.request_diagnostics_batch_via_lsp(uris)? {
            return Ok(results);
        }

        uris.iter()
            .map(|uri| {
                let diagnostics = self.request_diagnostics(uri.as_str())?;
                Ok((uri.clone(), diagnostics))
            })
            .collect()
    }

    pub(crate) fn request_diagnostics_full(
        &mut self,
        uri: &str,
    ) -> Result<DiagnosticFetch, String> {
        if self.supports_file_diagnostics_api()
            && self.can_use_api_for_uri(uri)
            && let Some(fetch) = self.request_diagnostics_full_via_file_api(uri)?
        {
            return fetch;
        }

        if self.supports_project_diagnostics_api()
            && self.can_use_api_for_uri(uri)
            && let Some(fetch) = self.request_diagnostics_full_via_project_api(uri)?
        {
            return fetch;
        }

        if let Some(fetch) = self.request_diagnostics_full_via_lsp(uri)? {
            return Ok(fetch);
        }

        let cached = self.cached_diagnostics(uri);
        if cached.is_empty() {
            return Err(cstr!(
                "Corsa diagnostics are unavailable for {uri}; no fresh diagnostics were produced"
            ));
        }

        Ok(DiagnosticFetch {
            diagnostics: cached,
            used_cache: true,
        })
    }

    fn request_diagnostics_batch_via_project_api(
        &mut self,
        uris: &[String],
    ) -> Result<Option<DiagnosticBatch>, String> {
        let report = match block_on(self.project_session()?.get_diagnostics_for_project()) {
            Ok(report) => report,
            Err(error) if corsa_diagnostics_error_is_unsupported(&error) => return Ok(None),
            Err(error) => {
                return Err(cstr!(
                    "Failed to request Corsa project diagnostics: {error}"
                ));
            }
        };
        Ok(Some(map_project_diagnostics(self, &report, uris)))
    }

    fn can_batch_with_project_diagnostics(&self, uris: &[String]) -> bool {
        self.supports_project_diagnostics_api()
            && uris.iter().all(|uri| {
                let uri = uri.as_str();
                self.can_use_api_for_uri(uri) && !self.document_texts.contains_key(uri)
            })
    }

    fn request_diagnostics_full_via_file_api(
        &mut self,
        uri: &str,
    ) -> Result<Option<Result<DiagnosticFetch, String>>, String> {
        let document_uri = self.session_document_uri(uri);
        if document_uri != uri {
            return self.request_materialized_diagnostics_full(uri, document_uri.as_str());
        }

        let response = block_on(
            self.project_session()?
                .get_diagnostics_for_file(uri_document_identifier(document_uri.as_str())),
        );
        let response = match response {
            Ok(response) => response,
            Err(error) if corsa_diagnostics_error_is_unsupported(&error) => return Ok(None),
            Err(error) => return Err(cstr!("Failed to request Corsa file diagnostics: {error}")),
        };
        Ok(Some(Ok(self.store_file_diagnostics(
            uri,
            self.remap_diagnostics(flatten_file_diagnostics(&response)),
        ))))
    }

    fn request_diagnostics_full_via_project_api(
        &mut self,
        uri: &str,
    ) -> Result<Option<Result<DiagnosticFetch, String>>, String> {
        let report = match block_on(self.project_session()?.get_diagnostics_for_project()) {
            Ok(report) => report,
            Err(error) if corsa_diagnostics_error_is_unsupported(&error) => return Ok(None),
            Err(error) => {
                return Err(cstr!(
                    "Failed to request Corsa project diagnostics: {error}"
                ));
            }
        };
        let diagnostics = report
            .files
            .iter()
            .find(|file| document_identifier_uri(&file.file) == uri)
            .map(flatten_file_diagnostics)
            .unwrap_or_default();
        Ok(Some(Ok(self.store_file_diagnostics(
            uri,
            self.remap_diagnostics(diagnostics),
        ))))
    }

    fn store_file_diagnostics(
        &mut self,
        uri: &str,
        diagnostics: Vec<Diagnostic>,
    ) -> DiagnosticFetch {
        self.diagnostics.insert(uri.into(), diagnostics.clone());
        DiagnosticFetch {
            diagnostics,
            used_cache: false,
        }
    }

    pub(super) fn cached_diagnostics(&self, uri: &str) -> Vec<Diagnostic> {
        self.diagnostics.get(uri).cloned().unwrap_or_default()
    }

    fn has_materialized_documents(&mut self, uris: &[String]) -> bool {
        uris.iter()
            .any(|uri| self.session_document_uri(uri.as_str()) != uri.as_str())
    }

    fn request_materialized_diagnostics_full(
        &mut self,
        external_uri: &str,
        document_uri: &str,
    ) -> Result<Option<Result<DiagnosticFetch, String>>, String> {
        let Some(diagnostics) = self.request_materialized_diagnostics(document_uri)? else {
            return Ok(None);
        };
        Ok(Some(Ok(
            self.store_file_diagnostics(external_uri, diagnostics)
        )))
    }

    fn request_diagnostics_batch_via_materialized_files(
        &mut self,
        uris: &[String],
    ) -> Result<Option<DiagnosticBatch>, String> {
        let mut pairs = Vec::with_capacity(uris.len());
        for uri in uris {
            pairs.push((uri.clone(), self.session_document_uri(uri.as_str())));
        }

        if self.supports_project_diagnostics_api()
            && let Some(results) =
                self.request_materialized_diagnostics_batch_via_project_api(&pairs)?
        {
            return Ok(Some(results));
        }

        let mut results = Vec::with_capacity(pairs.len());
        for (uri, document_uri) in pairs {
            let Some(diagnostics) = self.request_materialized_diagnostics(document_uri.as_str())?
            else {
                return Ok(None);
            };
            self.diagnostics.insert(uri.clone(), diagnostics.clone());
            results.push((uri, convert_diagnostics(&diagnostics)));
        }

        Ok(Some(results))
    }

    fn request_materialized_diagnostics(
        &mut self,
        document_uri: &str,
    ) -> Result<Option<Vec<Diagnostic>>, String> {
        if self.supports_file_diagnostics_api() {
            let response = block_on(
                self.project_session()?
                    .get_diagnostics_for_file(uri_document_identifier(document_uri)),
            );
            let response = match response {
                Ok(response) => response,
                Err(error) if diagnostics_api_error_is_unsupported(&error) => {
                    // Fall through to project diagnostics when file diagnostics are unavailable.
                    // Some runtimes expose only one of these endpoints.
                    let _ = error;
                    // Proceed to the project-diagnostics branch below.
                    return if self.supports_project_diagnostics_api() {
                        let report =
                            match block_on(self.project_session()?.get_diagnostics_for_project()) {
                                Ok(report) => report,
                                Err(error) if diagnostics_api_error_is_unsupported(&error) => {
                                    return Ok(None);
                                }
                                Err(error) => {
                                    return Err(cstr!(
                                        "Failed to request Corsa project diagnostics: {error}"
                                    ));
                                }
                            };
                        let diagnostics = report
                            .files
                            .iter()
                            .find(|file| document_identifier_uri(&file.file) == document_uri)
                            .map(flatten_file_diagnostics)
                            .unwrap_or_default();
                        Ok(Some(self.remap_diagnostics(diagnostics)))
                    } else {
                        Ok(None)
                    };
                }
                Err(error) => {
                    return Err(cstr!("Failed to request Corsa file diagnostics: {error}"));
                }
            };
            return Ok(Some(
                self.remap_diagnostics(flatten_file_diagnostics(&response)),
            ));
        }

        if self.supports_project_diagnostics_api() {
            let report = match block_on(self.project_session()?.get_diagnostics_for_project()) {
                Ok(report) => report,
                Err(error) if diagnostics_api_error_is_unsupported(&error) => {
                    return Ok(None);
                }
                Err(error) => {
                    return Err(cstr!(
                        "Failed to request Corsa project diagnostics: {error}"
                    ));
                }
            };
            let diagnostics = report
                .files
                .iter()
                .find(|file| document_identifier_uri(&file.file) == document_uri)
                .map(flatten_file_diagnostics)
                .unwrap_or_default();
            return Ok(Some(self.remap_diagnostics(diagnostics)));
        }

        Ok(None)
    }

    fn request_materialized_diagnostics_batch_via_project_api(
        &mut self,
        pairs: &[(String, String)],
    ) -> Result<Option<DiagnosticBatch>, String> {
        let report = match block_on(self.project_session()?.get_diagnostics_for_project()) {
            Ok(report) => report,
            Err(error) if corsa_diagnostics_error_is_unsupported(&error) => return Ok(None),
            Err(error) => {
                return Err(cstr!(
                    "Failed to request Corsa project diagnostics: {error}"
                ));
            }
        };
        let mut diagnostics_by_document: FxHashMap<_, _> = report
            .files
            .iter()
            .map(|file| {
                (
                    document_identifier_uri(&file.file),
                    self.remap_diagnostics(flatten_file_diagnostics(file)),
                )
            })
            .collect();

        Ok(Some(
            pairs
                .iter()
                .map(|(uri, document_uri)| {
                    let diagnostics = diagnostics_by_document
                        .remove(document_uri.as_str())
                        .unwrap_or_default();
                    self.diagnostics.insert(uri.clone(), diagnostics.clone());
                    (uri.clone(), convert_diagnostics(&diagnostics))
                })
                .collect(),
        ))
    }

    fn request_diagnostics_full_via_lsp(
        &mut self,
        uri: &str,
    ) -> Result<Option<DiagnosticFetch>, String> {
        let Some(mut results) = self.request_diagnostics_batch_via_lsp(&[uri.into()])? else {
            return Ok(None);
        };
        let Some((_, diagnostics)) = results.pop() else {
            return Ok(None);
        };
        let diagnostics = diagnostics
            .into_iter()
            .map(lsp_diagnostic_to_native)
            .collect::<Vec<_>>();
        Ok(Some(self.store_file_diagnostics(uri, diagnostics)))
    }

    fn request_diagnostics_batch_via_lsp(
        &mut self,
        uris: &[String],
    ) -> Result<Option<DiagnosticBatch>, String> {
        if uris.is_empty() {
            return Ok(Some(Vec::new()));
        }

        if uris.len() > LSP_DIAGNOSTICS_BATCH_CHUNK_SIZE {
            let mut results = Vec::with_capacity(uris.len());
            for chunk in uris.chunks(LSP_DIAGNOSTICS_BATCH_CHUNK_SIZE) {
                let Some(mut chunk_results) =
                    self.request_diagnostics_batch_via_lsp_chunk(chunk)?
                else {
                    return Ok(None);
                };
                results.append(&mut chunk_results);
            }
            return Ok(Some(results));
        }

        self.request_diagnostics_batch_via_lsp_chunk(uris)
    }

    fn request_diagnostics_batch_via_lsp_chunk(
        &mut self,
        uris: &[String],
    ) -> Result<Option<DiagnosticBatch>, String> {
        let mut attempts = 0;
        loop {
            match self.request_diagnostics_batch_via_lsp_chunk_once(uris) {
                Err(error)
                    if attempts < LSP_DIAGNOSTICS_BATCH_TRANSIENT_RETRIES
                        && super::lsp_transport_error_is_transient(&error) =>
                {
                    attempts += 1;
                    std::thread::sleep(Duration::from_millis(25));
                }
                result => return result,
            }
        }
    }

    fn request_diagnostics_batch_via_lsp_chunk_once(
        &mut self,
        uris: &[String],
    ) -> Result<Option<DiagnosticBatch>, String> {
        if uris.is_empty() {
            return Ok(Some(Vec::new()));
        }

        if self.has_project_session()
            && !self.materialized_project_session
            && uris.iter().any(|uri| {
                self.document_texts.contains_key(uri.as_str())
                    && file_uri_to_path(uri.as_str()).is_none_or(|path| !path.exists())
            })
        {
            self.activate_materialized_project_session()?;
        }

        let (documents, document_pairs) = self.editor_lsp_diagnostic_documents(uris)?;

        let mut results = Vec::with_capacity(document_pairs.len());
        for (external_uri, document_uri) in document_pairs {
            let report = match self.diagnostics_via_editor_lsp(document_uri.as_str(), &documents) {
                Ok(report) => report,
                Err(error) if diagnostics_api_error_is_unsupported(&error) => {
                    return Ok(None);
                }
                Err(error) => {
                    return Err(cstr!(
                        "Failed to request LSP diagnostics for {external_uri}: {error}"
                    ));
                }
            };

            let diagnostics = self.remap_diagnostics(extract_lsp_report_diagnostics(report));
            self.diagnostics
                .insert(external_uri.clone(), diagnostics.clone());
            results.push((external_uri.clone(), convert_diagnostics(&diagnostics)));
        }

        Ok(Some(results))
    }

    fn editor_lsp_diagnostic_documents(
        &mut self,
        uris: &[String],
    ) -> Result<(EditorLspDiagnosticDocuments, EditorLspDiagnosticPairs), String> {
        let mut documents = FxHashMap::default();
        let current_documents = self
            .document_texts
            .iter()
            .map(|(uri, text)| (uri.clone(), text.clone()))
            .collect::<Vec<_>>();

        for (uri, text) in current_documents {
            let document_uri = self.editor_lsp_diagnostic_document_uri(uri.as_str());
            documents.insert(document_uri, text);
        }

        let mut pairs = Vec::with_capacity(uris.len());
        for uri in uris {
            let document_uri = self.editor_lsp_diagnostic_document_uri(uri.as_str());
            let text = documents
                .get(document_uri.as_str())
                .cloned()
                .or_else(|| self.document_texts.get(uri.as_str()).cloned())
                .or_else(|| read_file_uri(document_uri.as_str()))
                .or_else(|| read_file_uri(uri.as_str()))
                .ok_or_else(|| cstr!("Failed to load document text for {uri}"))?;
            documents.entry(document_uri.clone()).or_insert(text);
            pairs.push((uri.clone(), document_uri));
        }

        Ok((documents, pairs))
    }

    fn editor_lsp_diagnostic_document_uri(&mut self, uri: &str) -> String {
        if self.materialized_project_session {
            self.session_document_uri(uri)
        } else {
            uri.into()
        }
    }
}

/// The corsa `ProjectSession` reports a missing diagnostics endpoint through
/// the typed `CorsaError::Unsupported` variant: corsa-bind raises it directly
/// when a scope is unavailable and normalizes "unknown method" RPC errors to
/// it as well. Gating the diagnostics fallback on the variant keeps the
/// capability handshake typed instead of sniffing the rendered message text.
fn corsa_diagnostics_error_is_unsupported(error: &CorsaError) -> bool {
    matches!(error, CorsaError::Unsupported(_))
}

/// Message-text fallback used only on the legacy LSP transport, whose
/// `textDocument/diagnostic` errors arrive as an opaque `String` with no typed
/// variant to match against.
fn diagnostics_api_is_unsupported(error: &str) -> bool {
    error.contains("unknown API method")
        || error.contains("method not found")
        || error.contains("Unsupported")
        || error.contains("unsupported")
        || error.contains("not supported")
}

fn diagnostics_api_error_is_unsupported(error: &impl std::fmt::Display) -> bool {
    diagnostics_api_is_unsupported(cstr!("{error}").as_str())
}

fn extract_lsp_report_diagnostics(report: DocumentDiagnosticReportResult) -> Vec<Diagnostic> {
    match report {
        DocumentDiagnosticReportResult::Report(DocumentDiagnosticReport::Full(report)) => {
            report.full_document_diagnostic_report.items
        }
        DocumentDiagnosticReportResult::Report(DocumentDiagnosticReport::Unchanged(_)) => {
            Vec::new()
        }
        DocumentDiagnosticReportResult::Partial(_) => Vec::new(),
    }
}

fn lsp_diagnostic_to_native(diagnostic: LspDiagnostic) -> Diagnostic {
    Diagnostic {
        range: lsp_types::Range::new(
            lsp_types::Position::new(
                diagnostic.range.start.line,
                diagnostic.range.start.character,
            ),
            lsp_types::Position::new(diagnostic.range.end.line, diagnostic.range.end.character),
        ),
        severity: diagnostic.severity.and_then(lsp_severity_from_i32),
        code: diagnostic.code.map(json_code_to_lsp_code),
        code_description: None,
        source: diagnostic.source.map(|source| source.into()),
        message: diagnostic.message.into(),
        related_information: None,
        tags: None,
        data: None,
    }
}

fn lsp_severity_from_i32(severity: i32) -> Option<lsp_types::DiagnosticSeverity> {
    match severity {
        1 => Some(lsp_types::DiagnosticSeverity::ERROR),
        2 => Some(lsp_types::DiagnosticSeverity::WARNING),
        3 => Some(lsp_types::DiagnosticSeverity::INFORMATION),
        4 => Some(lsp_types::DiagnosticSeverity::HINT),
        _ => None,
    }
}

fn json_code_to_lsp_code(code: serde_json::Value) -> lsp_types::NumberOrString {
    match code {
        serde_json::Value::Number(number) => {
            if let Some(value) = number.as_i64() {
                lsp_types::NumberOrString::Number(value as i32)
            } else {
                lsp_types::NumberOrString::String(cstr!("{number}").into())
            }
        }
        serde_json::Value::String(string) => lsp_types::NumberOrString::String(string),
        other => lsp_types::NumberOrString::String(cstr!("{other}").into()),
    }
}

fn read_file_uri(uri: &str) -> Option<String> {
    let path = file_uri_to_path(uri)?;
    std::fs::read_to_string(path).ok().map(Into::into)
}

#[cfg(test)]
mod tests;
