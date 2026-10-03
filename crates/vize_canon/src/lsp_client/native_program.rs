//! Fresh complete reports for the sealed native Program consumer only.

use super::CorsaProjectClient;
use lsp_types::{
    DocumentDiagnosticReport, DocumentDiagnosticReportKind, DocumentDiagnosticReportResult,
    RelatedFullDocumentDiagnosticReport,
};
use vize_l0::{String, cstr};

impl CorsaProjectClient {
    pub(crate) fn request_native_program_diagnostics_full(
        &mut self,
        uri: &str,
    ) -> Result<RelatedFullDocumentDiagnosticReport, String> {
        if self.closed
            || !self.materialized_project_session
            || !self.document_texts.contains_key(uri)
        {
            return Err(cstr!(
                "Native Program diagnostics require its open configured document"
            ));
        }
        let (documents, pairs) = self.editor_lsp_diagnostic_documents(&[uri.into()])?;
        let [(external_uri, document_uri)] = pairs.as_slice() else {
            return Err(cstr!(
                "Native Program diagnostics require exactly one document identity"
            ));
        };
        if external_uri.as_str() != uri || !documents.contains_key(document_uri.as_str()) {
            return Err(cstr!(
                "Native Program diagnostics lost the open document identity"
            ));
        }
        let report = self.diagnostics_via_editor_lsp(document_uri.as_str(), &documents)?;
        let full = complete_report(report)?;
        // Keep the actual typed payload, including opaque data and foreign URIs.
        // Compatibility remapping/narrowing belongs to the legacy fetch path.
        self.diagnostics.insert(
            uri.into(),
            full.full_document_diagnostic_report.items.clone(),
        );
        Ok(full)
    }
}

fn complete_report(
    report: DocumentDiagnosticReportResult,
) -> Result<RelatedFullDocumentDiagnosticReport, String> {
    let DocumentDiagnosticReportResult::Report(DocumentDiagnosticReport::Full(full)) = report
    else {
        return Err(cstr!(
            "Native Program diagnostics require a fresh complete Full report"
        ));
    };
    if full.related_documents.as_ref().is_some_and(|documents| {
        documents
            .values()
            .any(|report| matches!(report, DocumentDiagnosticReportKind::Unchanged(_)))
    }) {
        return Err(cstr!(
            "Native Program related diagnostics require complete Full reports"
        ));
    }
    Ok(full)
}

#[cfg(test)]
mod tests {
    use super::{CorsaProjectClient, complete_report};
    use lsp_types::DocumentDiagnosticReportResult;

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    #[test]
    fn full_report_keeps_every_typed_field_and_foreign_opaque_uri() -> TestResult {
        let input = serde_json::json!({
            "kind":"full", "resultId":"actual-report",
            "items":[{
                "range":{"start":{"line":1,"character":2},"end":{"line":1,"character":3}},
                "severity":1,"code":2339,"source":"ts","message":"complete payload",
                "codeDescription":{"href":"file:///actual-project/code.ts"},
                "tags":[1,2], "data":{"uri":"file:///actual-project/opaque.ts","nested":[1,true]},
                "relatedInformation":[{"location":{"uri":"file:///actual-project/related.ts",
                    "range":{"start":{"line":3,"character":4},"end":{"line":3,"character":5}}},
                    "message":"original related payload"}]
            }],
            "relatedDocuments":{"file:///actual-project/foreign.ts":{"kind":"full","items":[]}}
        });
        let typed: DocumentDiagnosticReportResult = serde_json::from_value(input)?;
        let original = serde_json::to_value(&typed)?;
        let full = complete_report(typed)?;
        let retained =
            DocumentDiagnosticReportResult::Report(lsp_types::DocumentDiagnosticReport::Full(full));
        if serde_json::to_value(retained)? != original {
            return Err("complete native report changed".into());
        }
        Ok(())
    }

    #[test]
    fn unchanged_partial_missing_and_related_unchanged_cannot_synthesize_success() -> TestResult {
        for input in [
            serde_json::json!({"kind":"unchanged","resultId":"previous"}),
            serde_json::json!({"relatedDocuments":{}}),
            serde_json::json!({"kind":"full","items":[],"relatedDocuments":{
                "file:///actual-project/foreign.ts":{"kind":"unchanged","resultId":"previous"}
            }}),
        ] {
            let report: DocumentDiagnosticReportResult = serde_json::from_value(input)?;
            if complete_report(report).is_ok() {
                return Err("incomplete native report admitted".into());
            }
        }
        for input in [
            serde_json::Value::Null,
            serde_json::json!({"kind":"full"}),
            serde_json::json!({"kind":"full","items":null}),
        ] {
            if let Ok(report) = serde_json::from_value::<DocumentDiagnosticReportResult>(input)
                && complete_report(report).is_ok()
            {
                return Err("missing native report admitted".into());
            }
        }
        Ok(())
    }

    #[test]
    fn missing_live_document_refuses_before_transport_or_cache() -> TestResult {
        let mut client =
            CorsaProjectClient::empty_for_test(std::path::PathBuf::from("/actual-project"));
        client.materialized_project_session = true;
        if client
            .request_native_program_diagnostics_full("file:///actual-project/missing.ts")
            .is_ok()
        {
            return Err("missing native document admitted".into());
        }
        if !client.diagnostics.is_empty() {
            return Err("missing native document cached".into());
        }
        Ok(())
    }
}
