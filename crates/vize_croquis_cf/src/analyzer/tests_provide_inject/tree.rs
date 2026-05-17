use super::*;

#[test]
fn test_provide_inject_multiple_levels() {
    use crate::diagnostics::CrossFileDiagnosticKind;

    let mut analyzer =
        CrossFileAnalyzer::new(CrossFileOptions::default().with_provide_inject(true));

    // Grandparent provides 'globalState'
    let mut gp_analyzer = vize_croquis::Analyzer::with_options(AnalyzerOptions::full());
    gp_analyzer.analyze_script_setup(
        r#"import { provide } from 'vue'
provide('globalState', { app: 'test' })"#,
    );
    gp_analyzer
        .croquis_mut()
        .used_components
        .insert(vize_carton::CompactString::new("Parent"));
    let gp_analysis = gp_analyzer.finish();

    // Parent doesn't provide/inject anything, just passes through
    let mut parent_analyzer = vize_croquis::Analyzer::with_options(AnalyzerOptions::full());
    parent_analyzer.analyze_script_setup(r#"// No provide/inject"#);
    parent_analyzer
        .croquis_mut()
        .used_components
        .insert(vize_carton::CompactString::new("Child"));
    let parent_analysis = parent_analyzer.finish();

    // Child injects 'globalState' (from grandparent)
    let mut child_analyzer = vize_croquis::Analyzer::with_options(AnalyzerOptions::full());
    child_analyzer.analyze_script_setup(
        r#"import { inject } from 'vue'
const state = inject('globalState')"#,
    );
    let child_analysis = child_analyzer.finish();

    // Add files
    analyzer.add_file_with_analysis(Path::new("Grandparent.vue"), "", gp_analysis);
    analyzer.add_file_with_analysis(Path::new("Parent.vue"), "", parent_analysis);
    analyzer.add_file_with_analysis(Path::new("Child.vue"), "", child_analysis);

    // Rebuild edges
    analyzer.rebuild_component_edges();

    // Run analysis
    let result = analyzer.analyze();

    // Should have 1 match (globalState from Grandparent to Child)
    assert_eq!(
        result.provide_inject_matches.len(),
        1,
        "Should have 1 match for globalState"
    );

    // No errors
    let errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.is_error())
        .filter(|d| {
            matches!(
                d.kind,
                CrossFileDiagnosticKind::UnmatchedInject { .. }
                    | CrossFileDiagnosticKind::UnusedProvide { .. }
            )
        })
        .collect();
    assert_eq!(errors.len(), 0, "Should have no provide/inject errors");
}

#[test]
fn test_provide_inject_tree_keeps_pass_through_component() {
    let mut analyzer =
        CrossFileAnalyzer::new(CrossFileOptions::default().with_provide_inject(true));

    let mut app_analyzer = vize_croquis::Analyzer::with_options(AnalyzerOptions::full());
    app_analyzer.analyze_script_setup(
        r#"import { provide, reactive } from 'vue'
const state = reactive({ count: 0 })
provide('state', state)"#,
    );
    app_analyzer
        .croquis_mut()
        .used_components
        .insert(vize_carton::CompactString::new("Middle"));
    let app_analysis = app_analyzer.finish();

    let mut middle_analyzer = vize_croquis::Analyzer::with_options(AnalyzerOptions::full());
    middle_analyzer.analyze_script_setup("// pass-through component");
    middle_analyzer
        .croquis_mut()
        .used_components
        .insert(vize_carton::CompactString::new("Leaf"));
    let middle_analysis = middle_analyzer.finish();

    let mut leaf_analyzer = vize_croquis::Analyzer::with_options(AnalyzerOptions::full());
    leaf_analyzer.analyze_script_setup(
        r#"import { inject } from 'vue'
const state = inject('state')"#,
    );
    let leaf_analysis = leaf_analyzer.finish();

    analyzer.add_file_with_analysis(Path::new("App.vue"), "", app_analysis);
    analyzer.add_file_with_analysis(Path::new("Middle.vue"), "", middle_analysis);
    analyzer.add_file_with_analysis(Path::new("Leaf.vue"), "", leaf_analysis);
    analyzer.rebuild_component_edges();

    let result = analyzer.analyze();
    assert_eq!(result.provide_inject_matches.len(), 1);
    assert_eq!(
        result.provide_inject_matches[0].path.len(),
        3,
        "match path should include provider, pass-through, and consumer"
    );

    let tree = result
        .provide_inject_tree
        .as_ref()
        .expect("tree should be built");
    assert_eq!(tree.roots.len(), 1);
    assert_eq!(tree.roots[0].children.len(), 1);
    assert_eq!(tree.roots[0].children[0].children.len(), 1);
    assert_eq!(tree.roots[0].children[0].children[0].injects.len(), 1);
}

#[test]
fn test_shared_child_resolves_each_parent_provider_context() {
    use crate::diagnostics::CrossFileDiagnosticKind;

    let mut analyzer =
        CrossFileAnalyzer::new(CrossFileOptions::default().with_provide_inject(true));

    analyzer.add_file_with_analysis(
        Path::new("App.vue"),
        "",
        script_analysis("// renders both parents", &["ParentA", "ParentB"]),
    );
    let parent_a = analyzer.add_file_with_analysis(
        Path::new("ParentA.vue"),
        "",
        script_analysis(
            r#"import { provide, ref } from 'vue'
const theme = ref('dark')
provide('theme', theme)"#,
            &["Child"],
        ),
    );
    let parent_b = analyzer.add_file_with_analysis(
        Path::new("ParentB.vue"),
        "",
        script_analysis(
            r#"import { provide, ref } from 'vue'
const theme = ref('light')
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
    let mut theme_matches = result
        .provide_inject_matches
        .iter()
        .filter(|provider_match| provider_match.key == "theme")
        .collect::<Vec<_>>();
    theme_matches.sort_by_key(|provider_match| provider_match.provider.as_u32());

    assert_eq!(
        theme_matches.len(),
        2,
        "a reused child should be matched in each parent render context"
    );
    assert!(
        theme_matches
            .iter()
            .any(|provider_match| provider_match.provider == parent_a)
    );
    assert!(
        theme_matches
            .iter()
            .any(|provider_match| provider_match.provider == parent_b)
    );
    assert!(
        theme_matches
            .iter()
            .all(|provider_match| provider_match.consumer == child)
    );
    assert!(result.diagnostics.iter().all(|diagnostic| {
        !matches!(
            diagnostic.kind,
            CrossFileDiagnosticKind::UnmatchedInject { .. }
                | CrossFileDiagnosticKind::UnusedProvide { .. }
        )
    }));

    let tree = result
        .provide_inject_tree
        .as_ref()
        .expect("tree should be built");
    assert_eq!(tree.roots.len(), 2);
    assert!(tree.roots.iter().all(|root| {
        root.children.len() == 1
            && root.children[0].file_id == child
            && root.children[0].injects.len() == 1
    }));
}
