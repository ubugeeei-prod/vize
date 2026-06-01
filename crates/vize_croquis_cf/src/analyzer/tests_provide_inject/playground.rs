use super::*;

#[test]
fn test_playground_style_provide_inject() {
    // This test mimics the playground's exact setup (without template parsing)
    use crate::diagnostics::CrossFileDiagnosticKind;

    let mut analyzer = CrossFileAnalyzer::new(
        CrossFileOptions::default()
            .with_provide_inject(true)
            .with_fallthrough_attrs(true)
            .with_component_emits(true)
            .with_reactivity_tracking(true),
    );

    // App.vue - provides 'theme' and 'user', uses ParentComponent
    let app_script = r#"import { provide, ref } from 'vue'
import ParentComponent from './ParentComponent.vue'

const theme = ref('dark')
provide('theme', theme)
provide('user', { name: 'John', id: 1 })"#;

    let mut app_analyzer = vize_croquis::Analyzer::with_options(AnalyzerOptions::full());
    app_analyzer.analyze_script_setup(app_script);
    // Simulate template analysis adding used component
    app_analyzer
        .croquis_mut()
        .used_components
        .insert(vize_carton::CompactString::new("ParentComponent"));
    let app_analysis = app_analyzer.finish();

    // Debug: check used_components
    eprintln!(
        "App.vue used_components: {:?}",
        app_analysis.used_components
    );

    // ParentComponent.vue - injects 'theme' and 'user', uses ChildComponent
    let parent_script = r#"import { inject, ref, onMounted } from 'vue'
import ChildComponent from './ChildComponent.vue'

const theme = inject('theme')
const { name } = inject('user')"#;

    let mut parent_analyzer = vize_croquis::Analyzer::with_options(AnalyzerOptions::full());
    parent_analyzer.analyze_script_setup(parent_script);
    parent_analyzer
        .croquis_mut()
        .used_components
        .insert(vize_carton::CompactString::new("ChildComponent"));
    let parent_analysis = parent_analyzer.finish();

    eprintln!(
        "ParentComponent.vue used_components: {:?}",
        parent_analysis.used_components
    );

    // ChildComponent.vue - no provide/inject
    let child_script = r#"const emit = defineEmits(['change'])"#;
    let mut child_analyzer = vize_croquis::Analyzer::with_options(AnalyzerOptions::full());
    child_analyzer.analyze_script_setup(child_script);
    let child_analysis = child_analyzer.finish();

    // Add files
    analyzer.add_file_with_analysis(Path::new("App.vue"), app_script, app_analysis);
    analyzer.add_file_with_analysis(
        Path::new("ParentComponent.vue"),
        parent_script,
        parent_analysis,
    );
    analyzer.add_file_with_analysis(
        Path::new("ChildComponent.vue"),
        child_script,
        child_analysis,
    );

    // Rebuild edges
    analyzer.rebuild_component_edges();

    // Debug: check graph edges
    eprintln!("Graph nodes: {}", analyzer.graph().nodes().count());
    for node in analyzer.graph().nodes() {
        eprintln!(
            "  {} (component_name={:?}): imports={:?}",
            node.path, node.component_name, node.imports
        );
    }

    // Run analysis
    let result = analyzer.analyze();

    eprintln!("Diagnostics count: {}", result.diagnostics.len());
    for d in &result.diagnostics {
        eprintln!("  - {:?}: {}", d.kind, d.message);
    }

    // Should have provide/inject matches (theme and user)
    assert!(
        !result.provide_inject_matches.is_empty(),
        "Should have at least 1 match (theme), got: {:?}",
        result.provide_inject_matches
    );

    // Check for unmatched inject errors - should have none for 'theme'
    let unmatched_theme: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| matches!(&d.kind, CrossFileDiagnosticKind::UnmatchedInject { key } if key == "theme"))
        .collect();
    assert_eq!(
        unmatched_theme.len(),
        0,
        "Should have no unmatched inject for 'theme'"
    );
}

#[test]
fn test_provide_inject_with_component_usage_edge() {
    use crate::diagnostics::CrossFileDiagnosticKind;

    let mut analyzer =
        CrossFileAnalyzer::new(CrossFileOptions::default().with_provide_inject(true));

    // App.vue provides 'theme' and 'user'
    // App uses Child component in template (simulated via used_components)
    let mut app_analyzer = vize_croquis::Analyzer::with_options(AnalyzerOptions::full());
    app_analyzer.analyze_script_setup(
        r#"import { provide, ref } from 'vue'

const theme = ref('dark')
const user = ref({ name: 'Test' })

provide('theme', theme)
provide('user', user)"#,
    );
    // Manually add used component (normally from template analysis)
    app_analyzer
        .croquis_mut()
        .used_components
        .insert(vize_carton::CompactString::new("Child"));
    let app_analysis = app_analyzer.finish();

    // Child.vue injects 'theme' and 'user'
    let mut child_analyzer = vize_croquis::Analyzer::with_options(AnalyzerOptions::full());
    child_analyzer.analyze_script_setup(
        r#"import { inject } from 'vue'

const theme = inject('theme')
const user = inject('user')"#,
    );
    let child_analysis = child_analyzer.finish();

    // Add files with pre-computed analysis
    let _app_id =
        analyzer.add_file_with_analysis(Path::new("App.vue"), "script content", app_analysis);
    let _child_id =
        analyzer.add_file_with_analysis(Path::new("Child.vue"), "script content", child_analysis);

    // Rebuild component edges (App uses Child)
    analyzer.rebuild_component_edges();

    // Run analysis
    let result = analyzer.analyze();

    // Should have 2 provide/inject matches
    assert_eq!(
        result.provide_inject_matches.len(),
        2,
        "Should have 2 matches (theme and user)"
    );

    // Should have NO unmatched inject errors
    let unmatched_inject_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| matches!(d.kind, CrossFileDiagnosticKind::UnmatchedInject { .. }))
        .filter(|d| d.is_error())
        .collect();
    assert_eq!(
        unmatched_inject_errors.len(),
        0,
        "Should have no unmatched inject errors, but got: {:?}",
        unmatched_inject_errors
            .iter()
            .map(|d| &d.message)
            .collect::<Vec<_>>()
    );

    // Should have NO unused provide warnings
    let unused_provide_warnings: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| matches!(d.kind, CrossFileDiagnosticKind::UnusedProvide { .. }))
        .collect();
    assert_eq!(
        unused_provide_warnings.len(),
        0,
        "Should have no unused provide warnings, but got: {:?}",
        unused_provide_warnings
            .iter()
            .map(|d| &d.message)
            .collect::<Vec<_>>()
    );
}
