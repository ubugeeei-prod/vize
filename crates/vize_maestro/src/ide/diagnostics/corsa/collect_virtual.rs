//! The editor's side of the one diagnostic post-pass (P4-5a).
//!
//! Corsa's answers for every virtual document projecting one SFC are decoded
//! into finished diagnostics and handed to
//! [`vize_canon::projection::assemble_diagnostics`] once; this module only
//! decodes LSP positions and renders the assembled result as LSP diagnostics.

use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, NumberOrString, Position, Range};
use vize_canon::ImportSourceMap;
use vize_canon::projection::{
    AssembledOrigin, AssemblyPolicy, AuthoredSource, FinishedDiagnostic, ProjectedDocument,
    assemble_diagnostics,
};
use vize_canon::virtual_ts::{ProjectionFeatures, ProjectionMapping, ProjectionMeta};
use vize_l0::line_index::LineBreaks;

use super::super::{VirtualTsResult, sources};
use super::message::strip_corsa_overlay_paths;

/// One virtual document synced to Corsa, ready for assembly.
pub(super) struct CorsaDocument {
    pub(super) code: String,
    pub(super) mapping: ProjectionMapping,
    pub(super) import_source_map: ImportSourceMap,
}

impl CorsaDocument {
    pub(super) fn from_result(result: VirtualTsResult) -> Self {
        let mut mapping = ProjectionMapping::from_spans(result.source_mappings);
        for span in result.diagnostic_mappings {
            mapping.push_with(
                span,
                ProjectionMeta {
                    features: ProjectionFeatures::DIAGNOSTICS,
                    ..Default::default()
                },
            );
        }
        Self {
            code: result.code,
            mapping,
            import_source_map: result.import_source_map,
        }
    }
}

/// A finished Corsa diagnostic; the payload keeps Corsa's own code spelling.
pub(super) type CorsaFinished = FinishedDiagnostic<Option<serde_json::Value>>;

/// Ask Corsa for one synced document's diagnostics, in generated coordinates.
pub(super) async fn fetch_finished_diagnostics(
    bridge: &std::sync::Arc<vize_canon::CorsaBridge>,
    virtual_uri: &str,
    document: &CorsaDocument,
    index: usize,
) -> Result<Vec<CorsaFinished>, vize_canon::CorsaBridgeError> {
    tracing::info!(
        "generated virtual ts ({} bytes), mappings_count={}",
        document.code.len(),
        document.mapping.len()
    );
    #[cfg(test)]
    super::super::assembly_parity_custody::native(
        virtual_uri,
        "native_query_started",
        || serde_json::json!({"generated":document.code,"mapping":vize_l0::cstr!("{:?}",document.mapping)}),
    );
    let result = bridge.get_diagnostics(virtual_uri).await;
    #[cfg(test)]
    super::super::assembly_parity_custody::native(virtual_uri, "native_query_returned", || {
        match &result {
            Ok(diagnostics) => serde_json::json!({"Ok":diagnostics}),
            Err(error) => serde_json::json!({"Err":vize_l0::cstr!("{error:?}")}),
        }
    });
    let corsa_diags = result?;
    tracing::info!(
        "corsa returned {} raw diagnostics for {}",
        corsa_diags.len(),
        virtual_uri
    );
    let decode_started = std::time::Instant::now();
    let finished: Vec<_> = corsa_diags
        .into_iter()
        .filter_map(|diag| finished_from_lsp(&document.code, diag, index))
        .collect();
    tracing::info!(
        "decoded {} Corsa diagnostics in {:?}",
        finished.len(),
        decode_started.elapsed()
    );
    Ok(finished)
}

/// Decode one LSP diagnostic of the synced (import-rewritten) document.
pub(super) fn finished_from_lsp(
    code: &str,
    diagnostic: vize_canon::corsa_bridge::LspDiagnostic,
    document: usize,
) -> Option<CorsaFinished> {
    let start = LineBreaks::Lsp.position_to_offset(
        code,
        diagnostic.range.start.line,
        diagnostic.range.start.character,
    )?;
    let end = LineBreaks::Lsp
        .position_to_offset(
            code,
            diagnostic.range.end.line,
            diagnostic.range.end.character,
        )
        .unwrap_or(start.saturating_add(1));
    Some(FinishedDiagnostic {
        document,
        start,
        end,
        code: diagnostic.code.as_ref().and_then(parse_code),
        severity: diagnostic.severity.map(|severity| match severity {
            1..=3 => severity,
            _ => 4,
        }),
        message: diagnostic.message.as_str().into(),
        payload: diagnostic.code,
    })
}

fn parse_code(code: &serde_json::Value) -> Option<u32> {
    match code {
        serde_json::Value::Number(number) => number.as_u64().and_then(|n| n.try_into().ok()),
        serde_json::Value::String(text) => text.trim_start_matches("TS").parse().ok(),
        _ => None,
    }
}

/// Assemble every document's finished diagnostics for the authored SFC.
pub(super) fn assemble_corsa_diagnostics(
    content: &str,
    documents: &[CorsaDocument],
    finished: Vec<CorsaFinished>,
) -> Vec<Diagnostic> {
    assemble_corsa_diagnostics_for_source(
        &AuthoredSource::vue(content),
        content,
        documents,
        finished,
    )
}

pub(super) fn assemble_script_corsa_diagnostics(
    content: &str,
    documents: &[CorsaDocument],
    finished: Vec<CorsaFinished>,
) -> Vec<Diagnostic> {
    assemble_corsa_diagnostics_for_source(
        &AuthoredSource::script(content),
        content,
        documents,
        finished,
    )
}

fn assemble_corsa_diagnostics_for_source(
    authored: &AuthoredSource<'_>,
    content: &str,
    documents: &[CorsaDocument],
    finished: Vec<CorsaFinished>,
) -> Vec<Diagnostic> {
    let projected: Vec<_> = documents
        .iter()
        .map(|document| ProjectedDocument {
            generated: &document.code,
            import_map: &document.import_source_map,
            mapping: Some(&document.mapping),
            tsx: false,
        })
        .collect();
    // An editor shows every unused declaration the checker reports; Corsa
    // tags them as faded hints unless the project turns them into errors.
    let policy = AssemblyPolicy {
        report_unused: true,
    };
    let assembly_started = std::time::Instant::now();
    let assembled = assemble_diagnostics(authored, &projected, finished, policy);
    tracing::info!(
        "assembled {} Corsa diagnostics in {:?}",
        assembled.len(),
        assembly_started.elapsed()
    );
    let rendered: Vec<_> = assembled
        .into_iter()
        .map(|assembled| {
            let (start_line, start_character) =
                LineBreaks::Lsp.offset_to_position(content, assembled.start.min(content.len()));
            let (end_line, end_character) =
                LineBreaks::Lsp.offset_to_position(content, assembled.end.min(content.len()));
            let code = match assembled.origin {
                AssembledOrigin::Checker(Some(raw))
                | AssembledOrigin::MissingVueImport(Some(raw))
                    if parse_code(&raw) == assembled.code =>
                {
                    Some(corsa_diagnostic_code(raw))
                }
                _ => assembled
                    .code
                    .map(|code| NumberOrString::Number(code as i32)),
            };
            Diagnostic {
                range: Range {
                    start: Position::new(start_line, start_character),
                    end: Position::new(end_line, end_character),
                },
                severity: assembled.severity.map(|severity| match severity {
                    1 => DiagnosticSeverity::ERROR,
                    2 => DiagnosticSeverity::WARNING,
                    3 => DiagnosticSeverity::INFORMATION,
                    _ => DiagnosticSeverity::HINT,
                }),
                code,
                source: Some(sources::TYPE_CHECKER.to_string()),
                message: strip_corsa_overlay_paths(&assembled.message),
                ..Default::default()
            }
        })
        .collect();
    tracing::info!("rendered {} Corsa diagnostics", rendered.len());
    rendered
}

pub(in crate::ide) fn corsa_diagnostic_code(code: serde_json::Value) -> NumberOrString {
    match code {
        serde_json::Value::Number(number) => number.as_i64().map_or_else(
            || NumberOrString::String(number.to_string()),
            |value| {
                i32::try_from(value).map_or_else(
                    |_| NumberOrString::String(number.to_string()),
                    NumberOrString::Number,
                )
            },
        ),
        serde_json::Value::String(code) => NumberOrString::String(code),
        other => NumberOrString::String(other.to_string()),
    }
}

#[cfg(test)]
mod tests {
    #![expect(clippy::string_slice, reason = "tests assert by panicking")]
    use super::{
        CorsaDocument, assemble_corsa_diagnostics, corsa_diagnostic_code, finished_from_lsp,
    };
    use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, NumberOrString, Position, Range};
    use vize_canon::ImportSourceMap;
    use vize_canon::corsa_bridge::{LspDiagnostic, LspPosition, LspRange};
    use vize_canon::virtual_ts::{ProjectionMapping, VizeMapping};

    const CONTENT: &str = "<script setup lang=\"ts\">\nimport Child from './Child.vue'\nconst count: string = 1\n</script>\n";
    const GENERATED: &str = "import Child from './Child.vue.ts';\nconst count: string = 1\n";
    const TS5097: &str = "An import path can only end with a '.ts' extension when 'allowImportingTsExtensions' is enabled.";

    /// One document whose script statements map onto the authored SFC.
    fn document() -> CorsaDocument {
        let import = CONTENT.find("import Child").unwrap();
        let count = CONTENT.find("const count").unwrap();
        let generated_count = GENERATED.find("const count").unwrap();
        CorsaDocument {
            code: GENERATED.into(),
            mapping: ProjectionMapping::from_spans(vec![
                VizeMapping::new(0..34, import..import + 31),
                VizeMapping::new(generated_count..generated_count + 23, count..count + 23),
            ]),
            import_source_map: ImportSourceMap::empty(),
        }
    }

    fn lsp(needle: &str, severity: u8, code: serde_json::Value, message: &str) -> LspDiagnostic {
        let start = GENERATED.find(needle).unwrap() as u32;
        let line = GENERATED[..start as usize].matches('\n').count() as u32;
        let line_start = GENERATED[..start as usize]
            .rfind('\n')
            .map_or(0, |index| index + 1) as u32;
        LspDiagnostic {
            range: LspRange {
                start: LspPosition {
                    line,
                    character: start - line_start,
                },
                end: LspPosition {
                    line,
                    character: start - line_start + needle.len() as u32,
                },
            },
            severity: Some(severity),
            code: Some(code),
            source: Some("ts".into()),
            message: message.into(),
            related_information: None,
        }
    }

    fn assemble(diagnostics: Vec<LspDiagnostic>) -> Vec<Diagnostic> {
        let finished = diagnostics
            .into_iter()
            .filter_map(|diagnostic| finished_from_lsp(GENERATED, diagnostic, 0))
            .collect();
        assemble_corsa_diagnostics(CONTENT, &[document()], finished)
    }

    fn at(needle: &str, code: NumberOrString, message: &str) -> Diagnostic {
        let start = CONTENT.find(needle).unwrap();
        let line = CONTENT[..start].matches('\n').count() as u32;
        let character = (start - CONTENT[..start].rfind('\n').map_or(0, |index| index + 1)) as u32;
        Diagnostic {
            range: Range {
                start: Position::new(line, character),
                end: Position::new(line, character + needle.len() as u32),
            },
            severity: Some(DiagnosticSeverity::ERROR),
            code: Some(code),
            source: Some(super::sources::TYPE_CHECKER.to_string()),
            message: message.to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn corsa_diagnostic_codes_preserve_lsp_number_and_string_shapes() {
        assert_eq!(
            corsa_diagnostic_code(serde_json::json!(2322)),
            NumberOrString::Number(2322),
        );
        assert_eq!(
            corsa_diagnostic_code(serde_json::json!("TS2322")),
            NumberOrString::String("TS2322".to_string()),
        );
    }

    #[test]
    fn assembled_diagnostics_keep_the_checker_code_spelling() {
        let message = "Type 'number' is not assignable to type 'string'.";
        assert_eq!(
            assemble(vec![lsp("count", 1, serde_json::json!(2322), message)]),
            [at("count", NumberOrString::Number(2322), message)]
        );
        assert_eq!(
            assemble(vec![lsp("count", 1, serde_json::json!("TS2322"), message)]),
            [at(
                "count",
                NumberOrString::String("TS2322".into()),
                message
            )]
        );
    }

    #[test]
    fn exact_duplicates_are_reported_once() {
        let message = "Type 'number' is not assignable to type 'string'.";
        let diagnostic = lsp("count", 1, serde_json::json!(2322), message);
        assert_eq!(
            assemble(vec![diagnostic.clone(), diagnostic]),
            [at("count", NumberOrString::Number(2322), message)]
        );
    }

    #[test]
    fn only_ts7044_hints_are_suppressed() {
        let message = "Parameter 'x' implicitly has an 'any' type, but a better type may be inferred from usage.";
        let assembled = assemble(vec![
            lsp("count", 4, serde_json::json!(7044), message),
            lsp("count", 1, serde_json::json!(7044), message),
        ]);
        assert_eq!(
            assembled,
            [at("count", NumberOrString::Number(7044), message)]
        );
    }

    #[test]
    fn vue_import_extension_diagnostics_are_suppressed_on_both_sides_of_the_projection() {
        // The generated `.vue.ts` spelling is the import rewriter's; the authored
        // `.vue` spelling is always allowed.
        assert_eq!(
            assemble(vec![
                lsp("'./Child.vue.ts'", 1, serde_json::json!(5097), TS5097),
                lsp("'./Child.vue", 1, serde_json::json!("TS5097"), TS5097),
            ]),
            []
        );
    }

    #[test]
    fn inference_witnesses_keep_diagnostics_without_claiming_editor_navigation() {
        use crate::ide::diagnostics::VirtualTsResult;
        use vize_canon::virtual_ts::{ProjectionFeatures, ProjectionMeta, VizeSubSpan};

        let owner = VizeMapping::new(0..8, 20..28);
        let inference = VizeMapping {
            gen_range: 10..18,
            src_range: 20..28,
            sub_spans: vec![VizeSubSpan {
                gen_range: 11..15,
                src_range: 22..26,
            }],
        };
        let mut mapping = ProjectionMapping::from_spans(vec![owner.clone()]);
        mapping.push_with(
            inference.clone(),
            ProjectionMeta {
                features: ProjectionFeatures::DIAGNOSTICS,
                ..Default::default()
            },
        );
        let result = VirtualTsResult::from_projection(
            "01234567890123456789".into(),
            mapping,
            ImportSourceMap::empty(),
        );
        assert_eq!(result.source_mappings, vec![owner]);
        assert_eq!(result.diagnostic_mappings, vec![inference]);
        let document = CorsaDocument::from_result(result);
        assert_eq!(document.mapping.len(), 2);
        assert_eq!(document.mapping.meta(0).features, ProjectionFeatures::ALL);
        assert_eq!(
            document.mapping.meta(1).features,
            ProjectionFeatures::DIAGNOSTICS
        );
        assert_eq!(
            document.mapping.diagnostic_range_to_authored(11, 15),
            Some((22, 26))
        );
    }
}
