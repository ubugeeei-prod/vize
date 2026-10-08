//! Producer origin and source-domain deduplication for issue #7907.
use std::path::Path;
use vize_carton::CompactString;
use vize_croquis::{
    Analyzer, AnalyzerOptions, Croquis, ScopeId, TemplateExpression, TemplateExpressionKind,
};
use vize_croquis_cf::{
    CrossFileAnalyzer, CrossFileDiagnosticKind, CrossFileOptions, DiagnosticSeverity,
    DiagnosticSource,
};

fn expression(start: u32) -> TemplateExpression {
    TemplateExpression {
        content: CompactString::new("window.innerWidth"),
        kind: TemplateExpressionKind::Interpolation,
        start,
        end: start + 17,
        scope_id: ScopeId::ROOT,
        vif_guard: None,
    }
}

fn analyze(analysis: Croquis) -> Vec<vize_croquis_cf::CrossFileDiagnostic> {
    let mut analyzer =
        CrossFileAnalyzer::new(CrossFileOptions::minimal().with_server_client_boundary(true));
    analyzer.add_file_with_analysis(Path::new("Collision.vue"), "", analysis);
    analyzer.analyze().diagnostics
}

fn assert_browser(
    diagnostic: &vize_croquis_cf::CrossFileDiagnostic,
    start: u32,
    source: DiagnosticSource,
) {
    assert_eq!(
        diagnostic.kind,
        CrossFileDiagnosticKind::BrowserApiInSsr {
            api: "window".into(),
            context: "Browser global".into()
        }
    );
    assert_eq!(diagnostic.severity, DiagnosticSeverity::Warning);
    assert_eq!(diagnostic.primary_file.as_u32(), 0);
    assert_eq!(diagnostic.primary_source, source);
    assert_eq!(diagnostic.primary_offset, start);
    assert_eq!(diagnostic.primary_end_offset, start);
    assert!(diagnostic.related_files.is_empty());
    assert_eq!(
        diagnostic.message,
        "Browser API used in potentially SSR context"
    );
    assert_eq!(
        diagnostic.suggestion.as_deref(),
        Some(
            "Wrap in onMounted() or guard with !import.meta.env.SSR, import.meta.client, or typeof window !== 'undefined'"
        )
    );
}

#[test]
fn equal_script_and_template_offsets_remain_distinct_but_same_source_duplicates_merge() {
    let mut analysis = Croquis::new();
    analysis.template_expressions.push(expression(15));
    analysis
        .setup_context
        .note_browser_global("window".into(), 15);
    analysis
        .setup_context
        .note_browser_global("window".into(), 15);
    let diagnostics = analyze(analysis);
    assert_eq!(diagnostics.len(), 2);
    assert_browser(&diagnostics[0], 15, DiagnosticSource::Template);
    assert_browser(&diagnostics[1], 15, DiagnosticSource::Script);
}

#[test]
fn template_offsets_do_not_inherit_script_client_only_context() {
    let script =
        "import { onMounted } from 'vue'; onMounted(() => { const width = window.innerWidth; });";
    let start = script.find("window").unwrap() as u32;
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    let mut analysis = analyzer.finish();
    analysis.template_expressions.push(expression(start));
    let diagnostics = analyze(analysis);
    assert_eq!(diagnostics.len(), 1);
    assert_browser(&diagnostics[0], start, DiagnosticSource::Template);
}
