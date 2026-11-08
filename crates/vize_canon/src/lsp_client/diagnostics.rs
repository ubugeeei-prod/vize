use super::{
    CorsaProjectClient, DiagnosticFetch, LspDiagnostic, PhasedDiagnostic,
    diagnostics_api::{document_identifier_uri, map_project_diagnostics, phased_file_diagnostics},
    session::uri_document_identifier,
    utils::convert_diagnostics,
};
use crate::file_uri::file_uri_to_path;
use corsa::{CorsaError, runtime::block_on};
use std::time::Duration;
use vize_l0::{FxHashMap, String, cstr};

type DiagnosticBatch = Vec<(String, Vec<LspDiagnostic>)>;
type EditorLspDiagnosticDocuments = FxHashMap<String, String>;
type EditorLspDiagnosticPairs = Vec<(String, String)>;
const LSP_DIAGNOSTICS_BATCH_CHUNK_SIZE: usize = 128;
const LSP_DIAGNOSTICS_BATCH_TRANSIENT_RETRIES: usize = 1;

mod batch;
mod lsp_report;
mod provenance;
#[cfg(test)]
pub(super) mod test_route;
pub(super) mod virtual_overlay_diagnostics;

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

    pub(crate) fn request_diagnostics_full(
        &mut self,
        uri: &str,
    ) -> Result<DiagnosticFetch, String> {
        virtual_overlay_diagnostics::ensure_materialized_project(self, std::iter::once(uri))?;

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

        self.request_lsp_diagnostics_or_cache(uri)
    }

    fn request_lsp_diagnostics_or_cache(&mut self, uri: &str) -> Result<DiagnosticFetch, String> {
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
            self.remap_phased_diagnostics(phased_file_diagnostics(&response)),
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
            .map(phased_file_diagnostics)
            .unwrap_or_default();
        Ok(Some(Ok(self.store_file_diagnostics(
            uri,
            self.remap_phased_diagnostics(diagnostics),
        ))))
    }

    fn store_file_diagnostics(
        &mut self,
        uri: &str,
        diagnostics: Vec<PhasedDiagnostic>,
    ) -> DiagnosticFetch {
        self.diagnostics.insert(uri.into(), diagnostics.clone());
        DiagnosticFetch {
            diagnostics,
            used_cache: false,
        }
    }

    pub(super) fn cached_diagnostics(&self, uri: &str) -> Vec<PhasedDiagnostic> {
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
    ) -> Result<Option<Vec<PhasedDiagnostic>>, String> {
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
                            .map(phased_file_diagnostics)
                            .unwrap_or_default();
                        Ok(Some(self.remap_phased_diagnostics(diagnostics)))
                    } else {
                        Ok(None)
                    };
                }
                Err(error) => {
                    return Err(cstr!("Failed to request Corsa file diagnostics: {error}"));
                }
            };
            return Ok(Some(
                self.remap_phased_diagnostics(phased_file_diagnostics(&response)),
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
                .map(phased_file_diagnostics)
                .unwrap_or_default();
            return Ok(Some(self.remap_phased_diagnostics(diagnostics)));
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
                    self.remap_phased_diagnostics(phased_file_diagnostics(file)),
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
            self.activate_editor_overlay_fallback()?;
        }

        let (documents, document_pairs) = self.editor_lsp_diagnostic_documents(uris)?;
        let requests = document_pairs
            .iter()
            .map(|(external_uri, document_uri)| {
                (external_uri.clone(), self.editor_document_uri(document_uri))
            })
            .collect::<Vec<_>>();
        // One bounded batch owns one immutable query view. Preparing it inside
        // the loop below would clone and compare the whole retained project for
        // every requested URI, turning an N-document batch into O(N²) host work.
        let reports = match self.request_with_editor_lsp_documents_recovery(&documents, |session| {
            requests
                .iter()
                .map(|(external_uri, document_uri)| {
                    session.diagnostics(document_uri).map_err(|error| {
                        #[cfg(not(unix))]
                        if error.as_str() == super::EXPLICIT_CONFIG_ATTACHMENT_UNSUPPORTED {
                            return error;
                        }
                        cstr!("Failed to request LSP diagnostics for {external_uri}: {error}")
                    })
                })
                .collect::<Result<Vec<_>, _>>()
        }) {
            Ok(reports) => reports,
            #[cfg(not(unix))]
            Err(error) if error.as_str() == super::EXPLICIT_CONFIG_ATTACHMENT_UNSUPPORTED => {
                return Err(error);
            }
            Err(error) if diagnostics_api_error_is_unsupported(&error) => return Ok(None),
            Err(error) => return Err(error),
        };

        let mut results = Vec::with_capacity(document_pairs.len());
        for ((external_uri, _), report) in document_pairs.into_iter().zip(reports) {
            let diagnostics = self
                .remap_phased_diagnostics(lsp_report::extract_diagnostics_with_provenance(report));
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
        let mut documents = self.editor_documents();
        let mut pairs = Vec::with_capacity(uris.len());
        for uri in uris {
            let document_uri = self.editor_document_uri(uri.as_str());
            let text = documents
                .get(document_uri.as_str())
                .cloned()
                .or_else(|| self.document_texts.get(uri.as_str()).cloned())
                .or_else(|| lsp_report::read_file_uri(document_uri.as_str()))
                .or_else(|| lsp_report::read_file_uri(uri.as_str()))
                .ok_or_else(|| cstr!("Failed to load document text for {uri}"))?;
            documents.entry(document_uri.clone()).or_insert(text);
            pairs.push((uri.clone(), document_uri));
        }

        Ok((documents, pairs))
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

#[cfg(test)]
mod tests;
