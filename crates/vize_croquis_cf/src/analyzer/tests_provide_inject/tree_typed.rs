use super::*;
use crate::diagnostics::CrossFileDiagnosticKind;

#[test]
fn test_branch_type_mismatches_are_aggregated_without_losing_provider_types() {
    let mut analyzer =
        CrossFileAnalyzer::new(CrossFileOptions::default().with_provide_inject(true));
    analyzer.add_file_with_analysis(
        Path::new("Child.vue"),
        "",
        script_analysis(
            "import { inject } from 'vue'; interface ExpectedTheme { mode: string }; const theme = inject<ExpectedTheme>('theme')",
            &[],
        ),
    );
    analyzer.add_file_with_analysis(
        Path::new("ParentB.vue"),
        "",
        script_analysis(
            "import { provide } from 'vue'; interface ThemeB { light: boolean }; provide('theme', {} as ThemeB)",
            &["Child"],
        ),
    );
    analyzer.add_file_with_analysis(
        Path::new("App.vue"),
        "",
        script_analysis("// renders both typed providers", &["ParentA", "ParentB"]),
    );
    analyzer.add_file_with_analysis(
        Path::new("ParentA.vue"),
        "",
        script_analysis(
            "import { provide } from 'vue'; interface ThemeA { dark: boolean }; provide('theme', {} as ThemeA)",
            &["Child"],
        ),
    );
    analyzer.rebuild_component_edges();

    let result = analyzer.analyze();
    assert_eq!(result.provide_inject_matches.len(), 2);
    assert!(
        result
            .provide_inject_matches
            .iter()
            .all(|provider_match| provider_match.type_match == Some(false))
    );
    let mismatch_diagnostics = result
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            matches!(
                diagnostic.kind,
                CrossFileDiagnosticKind::ProvideInjectTypeMismatch { .. }
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(mismatch_diagnostics.len(), 1);
    let mismatch = mismatch_diagnostics[0];
    let (key, provided_type, injected_type) = match &mismatch.kind {
        CrossFileDiagnosticKind::ProvideInjectTypeMismatch {
            key,
            provided_type,
            injected_type,
        } => (key, provided_type, injected_type),
        _ => unreachable!(),
    };
    assert_eq!(provided_type, "ThemeA | ThemeB");
    assert_eq!(injected_type, "ExpectedTheme");

    let related_providers = mismatch
        .related_files
        .iter()
        .map(|(file_id, _, _)| {
            analyzer
                .registry()
                .get(*file_id)
                .and_then(|entry| entry.component_name.as_deref())
                .expect("related provider should be registered")
        })
        .collect::<Vec<_>>();
    assert_eq!(related_providers, vec!["ParentA", "ParentB"]);
    let diagnostic_json = serde_json::json!({
        "key": key.as_str(),
        "providedType": provided_type.as_str(),
        "injectedType": injected_type.as_str(),
        "relatedProviders": related_providers,
    });
    assert_eq!(
        diagnostic_json,
        serde_json::json!({
            "key": "theme",
            "providedType": "ThemeA | ThemeB",
            "injectedType": "ExpectedTheme",
            "relatedProviders": ["ParentA", "ParentB"],
        })
    );

    let markdown = mismatch.to_markdown();
    assert!(markdown.contains("ThemeA | ThemeB"));
    assert!(markdown.contains("nearest provide() branches"));
}
