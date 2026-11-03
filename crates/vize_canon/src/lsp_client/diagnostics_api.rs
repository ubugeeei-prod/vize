use super::{CorsaProjectClient, LspDiagnostic, PhasedDiagnostic, utils::convert_diagnostics};
use crate::corsa_bridge::DiagnosticPhase;
use crate::file_uri::path_to_file_uri;
use corsa::api::{DocumentIdentifier, FileDiagnosticsResponse, ProjectDiagnosticsResponse};
#[cfg(test)]
use lsp_types::Diagnostic;
use std::path::Path;
use vize_l0::{FxHashMap, String};

pub(super) fn map_project_diagnostics(
    client: &mut CorsaProjectClient,
    response: &ProjectDiagnosticsResponse,
    uris: &[String],
) -> Vec<(String, Vec<LspDiagnostic>)> {
    let mut diagnostics_by_uri = FxHashMap::default();

    for file in &response.files {
        let uri = document_identifier_uri(&file.file);
        let diagnostics = phased_file_diagnostics(file);
        client.diagnostics.insert(uri.clone(), diagnostics.clone());
        diagnostics_by_uri.insert(uri, diagnostics);
    }

    uris.iter()
        .map(|uri| {
            let diagnostics = diagnostics_by_uri.remove(uri.as_str()).unwrap_or_default();
            (uri.clone(), convert_diagnostics(&diagnostics))
        })
        .collect()
}

pub(super) fn phased_file_diagnostics(response: &FileDiagnosticsResponse) -> Vec<PhasedDiagnostic> {
    let mut diagnostics = Vec::with_capacity(
        response.syntactic.len() + response.semantic.len() + response.suggestion.len(),
    );
    for (phase, rows) in [
        (DiagnosticPhase::Syntactic, &response.syntactic),
        (DiagnosticPhase::Semantic, &response.semantic),
        (DiagnosticPhase::Suggestion, &response.suggestion),
    ] {
        diagnostics.extend(
            rows.iter()
                .cloned()
                .map(|diagnostic| PhasedDiagnostic { diagnostic, phase }),
        );
    }
    diagnostics
}

#[cfg(test)]
fn flatten_file_diagnostics(response: &FileDiagnosticsResponse) -> Vec<Diagnostic> {
    phased_file_diagnostics(response)
        .into_iter()
        .map(|row| row.diagnostic)
        .collect()
}

pub(super) fn document_identifier_uri(document: &DocumentIdentifier) -> String {
    match document {
        DocumentIdentifier::Uri { uri } => uri.as_str().into(),
        DocumentIdentifier::FileName(path) if path.as_str().starts_with("file://") => {
            path.as_str().into()
        }
        DocumentIdentifier::FileName(path) => path_to_file_uri(Path::new(path.as_str())),
    }
}

#[cfg(test)]
mod tests {
    use super::{document_identifier_uri, flatten_file_diagnostics};
    use corsa::api::{DocumentIdentifier, FileDiagnosticsResponse};
    use lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range};

    #[test]
    fn diagnostics_api_groups_are_flattened_in_order() {
        let response = FileDiagnosticsResponse {
            file: DocumentIdentifier::from("/workspace/App.vue.ts"),
            syntactic: vec![diagnostic("syntactic")],
            semantic: vec![diagnostic("semantic")],
            suggestion: vec![diagnostic("suggestion")],
        };

        let diagnostics = flatten_file_diagnostics(&response);
        let messages: Vec<_> = diagnostics
            .iter()
            .map(|diagnostic| diagnostic.message.as_str())
            .collect();
        assert_eq!(messages, vec!["syntactic", "semantic", "suggestion"]);
    }

    #[test]
    fn api_phase_comes_from_the_owned_bucket_not_the_diagnostic_code() {
        use crate::corsa_bridge::DiagnosticPhase;
        let mut same_code = diagnostic("same diagnostic code");
        same_code.code = Some(lsp_types::NumberOrString::Number(1192));
        let response = FileDiagnosticsResponse {
            file: DocumentIdentifier::from("/workspace/App.vue.ts"),
            syntactic: vec![same_code.clone()],
            semantic: vec![same_code.clone()],
            suggestion: vec![same_code],
        };
        let diagnostics = super::phased_file_diagnostics(&response);
        assert_eq!(
            diagnostics.iter().map(|row| row.phase).collect::<Vec<_>>(),
            [
                DiagnosticPhase::Syntactic,
                DiagnosticPhase::Semantic,
                DiagnosticPhase::Suggestion
            ]
        );
        assert!(
            diagnostics
                .iter()
                .all(|row| row.diagnostic.code == Some(lsp_types::NumberOrString::Number(1192)))
        );
    }

    #[test]
    fn file_names_are_normalized_to_file_uris() {
        assert_eq!(
            document_identifier_uri(&DocumentIdentifier::from("/workspace/App.vue.ts")),
            "file:///workspace/App.vue.ts"
        );
    }

    #[test]
    fn file_names_are_percent_encoded_when_normalized_to_uris() {
        assert_eq!(
            document_identifier_uri(&DocumentIdentifier::from("/workspace/pages/[id].vue.ts")),
            "file:///workspace/pages/%5Bid%5D.vue.ts"
        );
    }

    fn diagnostic(message: &str) -> Diagnostic {
        Diagnostic {
            range: Range::new(Position::new(0, 0), Position::new(0, 1)),
            severity: Some(DiagnosticSeverity::ERROR),
            code: None,
            code_description: None,
            source: Some("ts".into()),
            message: message.into(),
            related_information: None,
            tags: None,
            data: None,
        }
    }
}
