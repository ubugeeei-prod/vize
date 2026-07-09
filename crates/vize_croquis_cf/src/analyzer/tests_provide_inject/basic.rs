use super::*;

// === Provide/Inject Tests ===
// NOTE: CrossFileAnalyzer.analyze_single_file doesn't parse SFC tags,
// so we use .ts extension to pass raw script content

#[test]
fn test_provide_inject_basic_match() {
    let mut analyzer =
        CrossFileAnalyzer::new(CrossFileOptions::default().with_provide_inject(true));

    // Parent provides 'state' (using .ts extension to pass raw script)
    analyzer.add_file(
        Path::new("Parent.ts"),
        r#"import { provide, reactive } from 'vue'
const state = reactive({ count: 0 })
provide('state', state)"#,
    );

    // Child injects 'state'
    analyzer.add_file(
        Path::new("Child.ts"),
        r#"import { inject } from 'vue'
const state = inject('state')"#,
    );

    let result = analyzer.analyze();

    // Both files should be analyzed
    assert_eq!(result.stats.files_analyzed, 2);
}

#[test]
fn test_provide_inject_with_type_assertion() {
    let mut analyzer =
        CrossFileAnalyzer::new(CrossFileOptions::default().with_provide_inject(true));

    // Child injects 'state' with type assertion
    analyzer.add_file(
        Path::new("Child.ts"),
        r#"import { inject } from 'vue'
const state = inject('state') as { count: number; user: { name: string } }"#,
    );

    let _result = analyzer.analyze();

    // Should detect the inject even with type assertion
    let child_analysis = analyzer.get_analysis(analyzer.registry().iter().next().unwrap().id);
    assert!(child_analysis.is_some());

    let analysis = child_analysis.unwrap();
    assert_eq!(analysis.provide_inject.injects().len(), 1);
    assert_eq!(
        analysis.provide_inject.injects()[0].key,
        vize_croquis::provide::ProvideKey::String(vize_carton::CompactString::new("state"))
    );
}

#[test]
fn test_provide_inject_with_satisfies() {
    let mut analyzer =
        CrossFileAnalyzer::new(CrossFileOptions::default().with_provide_inject(true));

    // Child injects 'theme' with satisfies
    analyzer.add_file(
        Path::new("Child.ts"),
        r#"import { inject } from 'vue'
const theme = inject('theme') satisfies string | undefined"#,
    );

    let _result = analyzer.analyze();

    let child_analysis = analyzer.get_analysis(analyzer.registry().iter().next().unwrap().id);
    assert!(child_analysis.is_some());

    let analysis = child_analysis.unwrap();
    assert_eq!(analysis.provide_inject.injects().len(), 1);
}

#[test]
fn test_provide_with_symbol_key() {
    let mut analyzer =
        CrossFileAnalyzer::new(CrossFileOptions::default().with_provide_inject(true));

    // Using Symbol as provide key
    analyzer.add_file(
        Path::new("Parent.ts"),
        r#"import { provide } from 'vue'
const ThemeKey = Symbol('theme')
provide(ThemeKey, 'dark')"#,
    );

    let _result = analyzer.analyze();

    let parent_analysis = analyzer.get_analysis(analyzer.registry().iter().next().unwrap().id);
    assert!(parent_analysis.is_some());

    let analysis = parent_analysis.unwrap();
    assert_eq!(analysis.provide_inject.provides().len(), 1);
}

#[test]
fn test_provide_inject_records_explicit_types() {
    let mut analyzer =
        CrossFileAnalyzer::new(CrossFileOptions::default().with_provide_inject(true));

    analyzer.add_file(
        Path::new("Typed.ts"),
        r#"import { inject, provide } from 'vue'
type Theme = { mode: 'dark' | 'light' }
const ThemeKey = Symbol('theme')
provide(ThemeKey, ({ mode: 'dark' } as Theme))
const theme = inject<Theme>(ThemeKey)"#,
    );

    let _result = analyzer.analyze();
    let analysis = analyzer
        .get_analysis(analyzer.registry().iter().next().unwrap().id)
        .unwrap();

    assert_eq!(
        analysis.provide_inject.provides()[0].value_type.as_deref(),
        Some("Theme")
    );
    assert_eq!(
        analysis.provide_inject.injects()[0]
            .expected_type
            .as_deref(),
        Some("Theme")
    );
}

#[test]
fn test_provide_inject_type_match_uses_explicit_types() {
    let mut analyzer =
        CrossFileAnalyzer::new(CrossFileOptions::default().with_provide_inject(true));

    analyzer.add_file_with_analysis(
        Path::new("Parent.vue"),
        "",
        script_analysis(
            r#"import { provide } from 'vue'
type Theme = { mode: string }
const ThemeKey = Symbol('theme')
provide(ThemeKey, ({ mode: 'dark' } as Theme))"#,
            &["Child"],
        ),
    );
    analyzer.add_file_with_analysis(
        Path::new("Child.vue"),
        "",
        script_analysis(
            r#"import { inject } from 'vue'
type Theme = { mode: string }
const ThemeKey = Symbol('theme')
const theme = inject<Theme>(ThemeKey)"#,
            &[],
        ),
    );
    analyzer.rebuild_component_edges();

    let result = analyzer.analyze();

    assert_eq!(result.provide_inject_matches.len(), 1);
    assert_eq!(result.provide_inject_matches[0].type_match, Some(true));
    assert!(result.diagnostics.iter().all(|diagnostic| !matches!(
        diagnostic.kind,
        crate::diagnostics::CrossFileDiagnosticKind::ProvideInjectTypeMismatch { .. }
    )));
}

#[test]
fn test_provide_inject_type_mismatch_reports_related_provider() {
    use crate::diagnostics::CrossFileDiagnosticKind;

    let mut analyzer =
        CrossFileAnalyzer::new(CrossFileOptions::default().with_provide_inject(true));

    let provider = analyzer.add_file_with_analysis(
        Path::new("Parent.vue"),
        "",
        script_analysis(
            r#"import { provide } from 'vue'
type Theme = { mode: string }
const ThemeKey = Symbol('theme')
provide(ThemeKey, ({ mode: 'dark' } as Theme))"#,
            &["Child"],
        ),
    );
    analyzer.add_file_with_analysis(
        Path::new("Child.vue"),
        "",
        script_analysis(
            r#"import { inject } from 'vue'
type ThemeRef = { mode: string }
const ThemeKey = Symbol('theme')
const theme = inject<ThemeRef>(ThemeKey)"#,
            &[],
        ),
    );
    analyzer.rebuild_component_edges();

    let result = analyzer.analyze();

    assert_eq!(result.provide_inject_matches.len(), 1);
    assert_eq!(result.provide_inject_matches[0].type_match, Some(false));

    let diagnostic = result
        .diagnostics
        .iter()
        .find(|diagnostic| {
            matches!(
                diagnostic.kind,
                CrossFileDiagnosticKind::ProvideInjectTypeMismatch { .. }
            )
        })
        .expect("type mismatch diagnostic");

    match &diagnostic.kind {
        CrossFileDiagnosticKind::ProvideInjectTypeMismatch {
            key,
            provided_type,
            injected_type,
        } => {
            assert_eq!(key.as_str(), "ThemeKey");
            assert_eq!(provided_type.as_str(), "Theme");
            assert_eq!(injected_type.as_str(), "ThemeRef");
        }
        other => panic!("unexpected diagnostic: {other:?}"),
    }
    assert_eq!(diagnostic.related_files.len(), 1);
    assert_eq!(diagnostic.related_files[0].0, provider);
}

#[test]
fn test_inject_with_default_value() {
    let mut analyzer =
        CrossFileAnalyzer::new(CrossFileOptions::default().with_provide_inject(true));

    // Child injects with default value
    analyzer.add_file(
        Path::new("Child.ts"),
        r#"import { inject } from 'vue'
const theme = inject('theme', 'light')"#,
    );

    let _result = analyzer.analyze();

    let child_analysis = analyzer.get_analysis(analyzer.registry().iter().next().unwrap().id);
    assert!(child_analysis.is_some());

    let analysis = child_analysis.unwrap();
    let injects = analysis.provide_inject.injects();
    assert_eq!(injects.len(), 1);
    assert!(injects[0].default_value.is_some());
}

#[test]
fn test_multiple_provides_and_injects() {
    let mut analyzer =
        CrossFileAnalyzer::new(CrossFileOptions::default().with_provide_inject(true));

    // Component with multiple provides and injects
    analyzer.add_file(
        Path::new("Mixed.ts"),
        r#"import { provide, inject, ref } from 'vue'

// Inject from ancestor
const theme = inject('theme', 'light')
const user = inject('user')

// Provide for descendants
const count = ref(0)
provide('count', count)
provide('config', { debug: true })"#,
    );

    let _result = analyzer.analyze();

    let analysis = analyzer
        .get_analysis(analyzer.registry().iter().next().unwrap().id)
        .unwrap();

    assert_eq!(analysis.provide_inject.provides().len(), 2);
    assert_eq!(analysis.provide_inject.injects().len(), 2);
}
