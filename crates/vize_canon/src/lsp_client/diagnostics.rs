use super::{
    diagnostics_api::{document_identifier_uri, flatten_file_diagnostics, map_project_diagnostics},
    session::uri_document_identifier,
    utils::convert_diagnostics,
    CorsaProjectClient, DiagnosticFetch, LspDiagnostic,
};
use crate::file_uri::{file_uri_to_path, path_to_file_uri};
use corsa::{
    jsonrpc::InboundEvent,
    lsp::{LspClient, LspSpawnConfig, VirtualDocument},
    runtime::block_on,
};
use lsp_types::{Diagnostic, DocumentDiagnosticReport, DocumentDiagnosticReportResult, Uri};
use std::{
    str::FromStr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};
use vize_carton::{cstr, FxHashMap, String};

type DiagnosticBatch = Vec<(String, Vec<LspDiagnostic>)>;

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
        if self.has_materialized_documents(uris) {
            if let Some(results) = self.request_diagnostics_batch_via_materialized_files(uris)? {
                return Ok(results);
            }
        }

        if self.can_batch_with_project_diagnostics(uris) {
            if let Some(results) = self.request_diagnostics_batch_via_project_api(uris)? {
                return Ok(results);
            }
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
        if self.supports_file_diagnostics_api() && self.can_use_api_for_uri(uri) {
            if let Some(fetch) = self.request_diagnostics_full_via_file_api(uri)? {
                return fetch;
            }
        }

        if self.supports_project_diagnostics_api() && self.can_use_api_for_uri(uri) {
            if let Some(fetch) = self.request_diagnostics_full_via_project_api(uri)? {
                return fetch;
            }
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
        let report = match block_on(self.session.get_diagnostics_for_project()) {
            Ok(report) => report,
            Err(error) if diagnostics_api_error_is_unsupported(&error) => return Ok(None),
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
            self.session
                .get_diagnostics_for_file(uri_document_identifier(document_uri.as_str())),
        );
        let response = match response {
            Ok(response) => response,
            Err(error) if diagnostics_api_error_is_unsupported(&error) => return Ok(None),
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
        let report = match block_on(self.session.get_diagnostics_for_project()) {
            Ok(report) => report,
            Err(error) if diagnostics_api_error_is_unsupported(&error) => return Ok(None),
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

        if self.supports_project_diagnostics_api() {
            if let Some(results) =
                self.request_materialized_diagnostics_batch_via_project_api(&pairs)?
            {
                return Ok(Some(results));
            }
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
                self.session
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
                        let report = match block_on(self.session.get_diagnostics_for_project()) {
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
            let report = match block_on(self.session.get_diagnostics_for_project()) {
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
        let report = match block_on(self.session.get_diagnostics_for_project()) {
            Ok(report) => report,
            Err(error) if diagnostics_api_error_is_unsupported(&error) => return Ok(None),
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

        let client = block_on(LspClient::spawn(
            LspSpawnConfig::new(self.executable.as_str()).with_cwd(self.cwd.clone()),
        ))
        .map_err(|error| cstr!("Failed to start Corsa LSP session: {error}"))?;
        let stop = Arc::new(AtomicBool::new(false));
        let responder = spawn_lsp_responder(client.clone(), stop.clone());

        let initialize_result = initialize_lsp_client(&client, &self.project_root);
        if let Err(error) = initialize_result {
            stop.store(true, Ordering::Relaxed);
            let _ = block_on(client.close());
            let _ = responder.join();
            return Err(error);
        }

        let overlay = client.overlay();
        let mut opened_documents = Vec::with_capacity(uris.len());
        for uri in uris {
            let document_uri = self.session_document_uri(uri.as_str());
            let text = self
                .document_texts
                .get(uri.as_str())
                .cloned()
                .or_else(|| read_file_uri(document_uri.as_str()))
                .or_else(|| read_file_uri(uri.as_str()))
                .ok_or_else(|| cstr!("Failed to load document text for {uri}"))?;
            let lsp_uri = Uri::from_str(document_uri.as_str())
                .map_err(|error| cstr!("Invalid LSP document URI {document_uri}: {error}"))?;
            let document = VirtualDocument::new(
                lsp_uri.clone(),
                language_id_for_uri(document_uri.as_str()),
                text,
            );
            overlay
                .open(document)
                .map_err(|error| cstr!("Failed to open LSP overlay for {document_uri}: {error}"))?;
            opened_documents.push((uri.clone(), lsp_uri));
        }

        let mut results = Vec::with_capacity(opened_documents.len());
        for (external_uri, lsp_uri) in &opened_documents {
            let report = match request_lsp_document_diagnostics(&client, lsp_uri) {
                Ok(report) => report,
                Err(error) if diagnostics_api_error_is_unsupported(&error) => {
                    cleanup_lsp_session(&overlay, &opened_documents, stop, responder, &client);
                    return Ok(None);
                }
                Err(error) => {
                    cleanup_lsp_session(&overlay, &opened_documents, stop, responder, &client);
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

        cleanup_lsp_session(&overlay, &opened_documents, stop, responder, &client);
        Ok(Some(results))
    }
}

fn diagnostics_api_is_unsupported(error: &str) -> bool {
    error.contains("unknown API method")
        || error.contains("method not found")
        || error.contains("Unsupported")
}

fn diagnostics_api_error_is_unsupported(error: &impl std::fmt::Display) -> bool {
    diagnostics_api_is_unsupported(cstr!("{error}").as_str())
}

fn initialize_lsp_client(client: &LspClient, project_root: &std::path::Path) -> Result<(), String> {
    struct InitializeRequest;

    impl lsp_types::request::Request for InitializeRequest {
        type Params = serde_json::Value;
        type Result = serde_json::Value;
        const METHOD: &'static str = "initialize";
    }

    struct InitializedNotification;

    impl lsp_types::notification::Notification for InitializedNotification {
        type Params = serde_json::Value;
        const METHOD: &'static str = "initialized";
    }

    let root_uri = path_to_file_uri(project_root);
    block_on(client.request::<InitializeRequest>(serde_json::json!({
        "processId": std::process::id(),
        "rootUri": root_uri,
        "capabilities": {
            "textDocument": {
                "publishDiagnostics": {},
                "diagnostic": {
                    "dynamicRegistration": false,
                    "relatedDocumentSupport": true,
                }
            },
            "workspace": {
                "diagnostic": {
                    "refreshSupport": true,
                }
            }
        }
    })))
    .map_err(|error| cstr!("Failed to initialize Corsa LSP session: {error}"))?;
    client
        .notify::<InitializedNotification>(serde_json::json!({}))
        .map_err(|error| cstr!("Failed to send LSP initialized notification: {error}"))?;
    Ok(())
}

fn request_lsp_document_diagnostics(
    client: &LspClient,
    uri: &Uri,
) -> Result<DocumentDiagnosticReportResult, String> {
    struct RawDocumentDiagnosticRequest;

    impl lsp_types::request::Request for RawDocumentDiagnosticRequest {
        type Params = serde_json::Value;
        type Result = DocumentDiagnosticReportResult;
        const METHOD: &'static str = "textDocument/diagnostic";
    }

    block_on(
        client.request::<RawDocumentDiagnosticRequest>(serde_json::json!({
            "textDocument": {
                "uri": uri,
            }
        })),
    )
    .map_err(|error| cstr!("{error}"))
}

fn spawn_lsp_responder(client: LspClient, stop: Arc<AtomicBool>) -> std::thread::JoinHandle<()> {
    let events = client.subscribe();
    std::thread::spawn(move || {
        while !stop.load(Ordering::Relaxed) {
            match events.recv_timeout(Duration::from_millis(50)) {
                Ok(InboundEvent::Request { id, method, .. }) => {
                    let response = match method.as_ref() {
                        "workspace/configuration" => serde_json::json!([]),
                        _ => serde_json::Value::Null,
                    };
                    let _ = client.respond(id, response);
                }
                Ok(_) => {}
                Err(_) => {}
            }
        }
    })
}

fn cleanup_lsp_session(
    _overlay: &corsa::lsp::LspOverlay,
    _opened_documents: &[(String, Uri)],
    stop: Arc<AtomicBool>,
    responder: std::thread::JoinHandle<()>,
    client: &LspClient,
) {
    stop.store(true, Ordering::Relaxed);
    let _ = block_on(client.close());
    let _ = responder.join();
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

fn language_id_for_uri(uri: &str) -> &'static str {
    if uri.ends_with(".tsx") || uri.ends_with(".jsx") {
        "typescriptreact"
    } else {
        "typescript"
    }
}
