use super::*;

#[test]
fn test_nearest_ancestor_provider_shadows_grandparent() {
    use crate::diagnostics::CrossFileDiagnosticKind;

    let mut analyzer =
        CrossFileAnalyzer::new(CrossFileOptions::default().with_provide_inject(true));

    let grandparent = analyzer.add_file_with_analysis(
        Path::new("Grandparent.vue"),
        "",
        script_analysis(
            r#"import { provide } from 'vue'
provide('theme', 'global')"#,
            &["Parent"],
        ),
    );
    let parent = analyzer.add_file_with_analysis(
        Path::new("Parent.vue"),
        "",
        script_analysis(
            r#"import { provide, ref } from 'vue'
const theme = ref('local')
provide('theme', theme)"#,
            &["Child"],
        ),
    );
    let child = analyzer.add_file_with_analysis(
        Path::new("Child.vue"),
        "",
        script_analysis(
            r#"import { inject } from 'vue'
const theme = inject('theme')"#,
            &[],
        ),
    );
    analyzer.rebuild_component_edges();

    let result = analyzer.analyze();
    let matches = result
        .provide_inject_matches
        .iter()
        .filter(|provider_match| provider_match.key == "theme")
        .collect::<Vec<_>>();

    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].provider, parent);
    assert_eq!(matches[0].consumer, child);
    assert_eq!(matches[0].path, vec![parent, child]);
    assert!(result.diagnostics.iter().any(|diagnostic| {
        diagnostic.primary_file == grandparent
            && matches!(
                diagnostic.kind,
                CrossFileDiagnosticKind::UnusedProvide { .. }
            )
    }));
}

#[test]
fn test_later_provide_in_same_component_wins_for_same_key() {
    use crate::diagnostics::CrossFileDiagnosticKind;

    let mut analyzer =
        CrossFileAnalyzer::new(CrossFileOptions::default().with_provide_inject(true));
    let parent_script = r#"import { provide, ref } from 'vue'
const firstTheme = 'plain'
const secondTheme = ref('reactive')
provide('theme', firstTheme)
provide('theme', secondTheme)"#;
    let earlier_offset = parent_script.find("provide('theme', firstTheme)").unwrap() as u32;
    let later_offset = parent_script.find("provide('theme', secondTheme)").unwrap() as u32;

    let parent = analyzer.add_file_with_analysis(
        Path::new("Parent.vue"),
        "",
        script_analysis(parent_script, &["Child"]),
    );
    let child = analyzer.add_file_with_analysis(
        Path::new("Child.vue"),
        "",
        script_analysis(
            r#"import { inject } from 'vue'
const theme = inject('theme')"#,
            &[],
        ),
    );
    analyzer.rebuild_component_edges();

    let result = analyzer.analyze();
    let provider_match = result
        .provide_inject_matches
        .iter()
        .find(|provider_match| provider_match.key == "theme")
        .expect("theme should match the nearest provider");

    assert_eq!(provider_match.provider, parent);
    assert_eq!(provider_match.consumer, child);
    assert_eq!(provider_match.provide_offset, later_offset);
    assert!(result.diagnostics.iter().any(|diagnostic| {
        diagnostic.primary_file == parent
            && diagnostic.primary_offset == earlier_offset
            && matches!(
                diagnostic.kind,
                CrossFileDiagnosticKind::UnusedProvide { .. }
            )
    }));
}

#[test]
fn test_sibling_provider_does_not_satisfy_inject() {
    use crate::diagnostics::CrossFileDiagnosticKind;

    let mut analyzer =
        CrossFileAnalyzer::new(CrossFileOptions::default().with_provide_inject(true));

    let provider = analyzer.add_file_with_analysis(
        Path::new("ProviderSibling.vue"),
        "",
        script_analysis(
            r#"import { provide, ref } from 'vue'
const theme = ref('dark')
provide('theme', theme)"#,
            &[],
        ),
    );
    let consumer = analyzer.add_file_with_analysis(
        Path::new("ConsumerSibling.vue"),
        "",
        script_analysis(
            r#"import { inject } from 'vue'
const theme = inject('theme')"#,
            &[],
        ),
    );
    analyzer.add_file_with_analysis(
        Path::new("App.vue"),
        "",
        script_analysis(
            "// siblings render in the same parent",
            &["ProviderSibling", "ConsumerSibling"],
        ),
    );
    analyzer.rebuild_component_edges();

    let result = analyzer.analyze();

    assert!(result.provide_inject_matches.is_empty());
    assert!(result.diagnostics.iter().any(|diagnostic| {
        diagnostic.primary_file == consumer
            && matches!(
                &diagnostic.kind,
                CrossFileDiagnosticKind::UnmatchedInject { key } if key == "theme"
            )
    }));
    assert!(result.diagnostics.iter().any(|diagnostic| {
        diagnostic.primary_file == provider
            && matches!(
                &diagnostic.kind,
                CrossFileDiagnosticKind::UnusedProvide { key } if key == "theme"
            )
    }));
}

#[test]
fn test_defaulted_inject_without_ancestor_provider_is_warning() {
    use crate::diagnostics::CrossFileDiagnosticKind;
    use crate::diagnostics::DiagnosticSeverity;

    let mut analyzer =
        CrossFileAnalyzer::new(CrossFileOptions::default().with_provide_inject(true));
    analyzer.add_file_with_analysis(
        Path::new("Child.vue"),
        "",
        script_analysis(
            r#"import { inject } from 'vue'
const theme = inject('theme', 'light')"#,
            &[],
        ),
    );

    let result = analyzer.analyze();
    let diagnostic = result
        .diagnostics
        .iter()
        .find(|diagnostic| {
            matches!(
                &diagnostic.kind,
                CrossFileDiagnosticKind::UnmatchedInject { key } if key == "theme"
            )
        })
        .expect("defaulted inject should still be reported");

    assert_eq!(diagnostic.severity, DiagnosticSeverity::Warning);
}
