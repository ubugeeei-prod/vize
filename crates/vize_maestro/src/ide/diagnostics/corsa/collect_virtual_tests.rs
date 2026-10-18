#![expect(clippy::string_slice, reason = "tests assert by panicking")]
use super::{CorsaDocument, assemble_corsa_diagnostics, corsa_diagnostic_code, finished_from_lsp};
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
    let message =
        "Parameter 'x' implicitly has an 'any' type, but a better type may be inferred from usage.";
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
