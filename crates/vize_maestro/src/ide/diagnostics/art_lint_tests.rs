//! CLI/editor parity for authored Art variant lint diagnostics (#7945).

use super::{DiagnosticService, LineIndex};
use crate::server::ServerState;
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, NumberOrString, Url};

const SOURCE: &str = include_str!("../../../../../tests/_fixtures/art-lsp-lint/Card.art.vue");

fn a11y(diagnostics: Vec<Diagnostic>) -> Vec<Diagnostic> {
    diagnostics
        .into_iter()
        .filter(|diagnostic| {
            matches!(&diagnostic.code, Some(NumberOrString::String(code)) if code.starts_with("a11y/"))
        })
        .collect()
}

#[test]
fn both_art_language_ids_publish_the_cli_variant_findings() {
    for language in ["art-vue", "vue"] {
        for unicode in [false, true] {
            let source = if unicode {
                SOURCE.replace("<a href", "😀<a href")
            } else {
                SOURCE.to_string()
            };
            let root = tempfile::tempdir().unwrap();
            let uri = Url::from_file_path(root.path().join("Card.art.vue")).unwrap();
            let state = ServerState::new();
            state.apply_lsp_initialization_options(Some(
                &serde_json::json!({"lint":true,"typecheck":false}),
            ));
            state.apply_initialize_workspace_folders(None, Some(root.path()));
            state
                .documents
                .open(uri.clone(), source.clone(), 1, language.into());

            let full = a11y(DiagnosticService::collect(&state, &uri));
            let lint_only = a11y(DiagnosticService::collect_lint_only(&state, &uri));
            assert_eq!(full, lint_only, "{language}, unicode={unicode}");
            assert_eq!(full.len(), 2, "{language}, unicode={unicode}");
            let cli = vize_patina::Linter::new()
                .with_enabled_rules(Some(vec![
                    "a11y/anchor-is-valid".into(),
                    "a11y/alt-text".into(),
                ]))
                .lint_sfc(&source, uri.path());
            assert_eq!(cli.diagnostics.len(), 2);
            let index = LineIndex::new(&source);
            for diagnostic in &full {
                assert_eq!(diagnostic.source.as_deref(), Some("vize/lint"));
                assert_eq!(diagnostic.severity, Some(DiagnosticSeverity::WARNING));
                let Some(NumberOrString::String(rule)) = &diagnostic.code else {
                    panic!("rule identifier missing");
                };
                let original = cli
                    .diagnostics
                    .iter()
                    .find(|item| item.rule_name == rule.as_str())
                    .unwrap();
                let (line, character) = index.line_col(original.start as usize);
                assert_eq!(
                    (
                        diagnostic.range.start.line,
                        diagnostic.range.start.character
                    ),
                    (line, character)
                );
                let (line, character) = index.line_col(original.end as usize);
                assert_eq!(
                    (diagnostic.range.end.line, diagnostic.range.end.character),
                    (line, character)
                );
                assert!(diagnostic.message.starts_with(original.message.as_str()));
                let expected = if rule == "a11y/anchor-is-valid" {
                    (7, if unicode { 11 } else { 9 })
                } else {
                    (8, 6)
                };
                assert_eq!(
                    (
                        diagnostic.range.start.line,
                        diagnostic.range.start.character
                    ),
                    expected
                );
            }
        }
    }
}

#[test]
fn art_template_diagnostics_obey_lint_disabled() {
    let state = ServerState::new();
    state.apply_lsp_initialization_options(Some(
        &serde_json::json!({"lint":false,"typecheck":false}),
    ));
    let uri = Url::parse("file:///project/Card.art.vue").unwrap();
    state
        .documents
        .open(uri.clone(), SOURCE.into(), 1, "art-vue".into());
    assert!(DiagnosticService::collect(&state, &uri).is_empty());
    assert!(DiagnosticService::collect_lint_only(&state, &uri).is_empty());
}
