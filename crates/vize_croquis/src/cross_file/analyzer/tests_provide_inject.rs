use super::{CrossFileAnalyzer, CrossFileOptions};
use crate::AnalyzerOptions;
use std::path::Path;

fn script_analysis(script: &str, used_components: &[&str]) -> crate::Croquis {
    let mut analyzer = crate::Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    for component in used_components {
        analyzer
            .croquis_mut()
            .used_components
            .insert(vize_carton::CompactString::new(*component));
    }
    analyzer.finish()
}

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
        crate::provide::ProvideKey::String(vize_carton::CompactString::new("state"))
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

#[test]
fn test_inject_object_destructure_pattern() {
    use crate::provide::InjectPattern;

    let mut analyzer =
        CrossFileAnalyzer::new(CrossFileOptions::default().with_reactivity_tracking(true));

    // Destructuring inject() loses reactivity
    analyzer.add_file(
        Path::new("Child.ts"),
        r#"import { inject } from 'vue'
const { count, name } = inject('state') as { count: number; name: string }"#,
    );

    let _result = analyzer.analyze();

    let analysis = analyzer
        .get_analysis(analyzer.registry().iter().next().unwrap().id)
        .unwrap();

    // Should detect the inject with ObjectDestructure pattern
    let injects = analysis.provide_inject.injects();
    assert_eq!(injects.len(), 1, "Should have 1 inject");
    match &injects[0].pattern {
        InjectPattern::ObjectDestructure(props) => {
            assert!(props.contains(&vize_carton::CompactString::new("count")));
            assert!(props.contains(&vize_carton::CompactString::new("name")));
        }
        _ => panic!(
            "Expected ObjectDestructure pattern, got {:?}",
            injects[0].pattern
        ),
    }
}

#[test]
fn test_inject_simple_pattern() {
    use crate::provide::InjectPattern;

    let mut analyzer =
        CrossFileAnalyzer::new(CrossFileOptions::default().with_provide_inject(true));

    // Simple inject without destructuring
    analyzer.add_file(
        Path::new("Child.ts"),
        r#"import { inject } from 'vue'
const state = inject('state')"#,
    );

    let _result = analyzer.analyze();

    let analysis = analyzer
        .get_analysis(analyzer.registry().iter().next().unwrap().id)
        .unwrap();

    let injects = analysis.provide_inject.injects();
    assert_eq!(injects.len(), 1);
    assert!(matches!(injects[0].pattern, InjectPattern::Simple));
}

#[test]
fn test_inject_destructure_with_type_assertion() {
    use crate::provide::InjectPattern;

    let mut analyzer =
        CrossFileAnalyzer::new(CrossFileOptions::default().with_reactivity_tracking(true));

    // Destructuring with TSAsExpression
    analyzer.add_file(
        Path::new("Child.ts"),
        r#"import { inject } from 'vue'
const { foo } = inject('data') as { foo: string }"#,
    );

    let _result = analyzer.analyze();

    let analysis = analyzer
        .get_analysis(analyzer.registry().iter().next().unwrap().id)
        .unwrap();

    let injects = analysis.provide_inject.injects();
    assert_eq!(injects.len(), 1);
    match &injects[0].pattern {
        InjectPattern::ObjectDestructure(props) => {
            assert!(props.contains(&vize_carton::CompactString::new("foo")));
        }
        _ => panic!("Expected ObjectDestructure pattern"),
    }
}

#[test]
fn test_inject_destructure_in_vue_sfc() {
    use crate::provide::InjectPattern;

    let mut analyzer =
        CrossFileAnalyzer::new(CrossFileOptions::default().with_reactivity_tracking(true));

    // Add Vue SFC script content (not full SFC - the caller should extract this)
    // The cross-file analyzer expects script content only for .vue files
    analyzer.add_file(
        Path::new("Child.vue"),
        r#"import { inject } from 'vue'

const { name } = inject('user') as { name: string; id: number }"#,
    );

    let _result = analyzer.analyze();

    let analysis = analyzer
        .get_analysis(analyzer.registry().iter().next().unwrap().id)
        .unwrap();

    let injects = analysis.provide_inject.injects();
    assert_eq!(injects.len(), 1, "Should have 1 inject");
    match &injects[0].pattern {
        InjectPattern::ObjectDestructure(props) => {
            assert!(
                props.contains(&vize_carton::CompactString::new("name")),
                "Should contain 'name' prop"
            );
        }
        other => panic!("Expected ObjectDestructure pattern, got {:?}", other),
    }
}

#[test]
fn test_inject_wrapped_in_torefs_tracks_inject_without_loss() {
    use crate::provide::InjectPattern;

    let mut analyzer =
        CrossFileAnalyzer::new(CrossFileOptions::default().with_reactivity_tracking(true));

    analyzer.add_file(
        Path::new("Child.ts"),
        r#"import { inject, toRefs } from 'vue'
const { count } = toRefs(inject('state') as { count: number })"#,
    );

    let result = analyzer.analyze();
    let analysis = analyzer
        .get_analysis(analyzer.registry().iter().next().unwrap().id)
        .unwrap();

    let injects = analysis.provide_inject.injects();
    assert_eq!(injects.len(), 1);
    assert!(matches!(injects[0].pattern, InjectPattern::Simple));
    assert!(
        result
            .diagnostics
            .iter()
            .all(|diagnostic| !matches!(
                diagnostic.kind,
                crate::cross_file::diagnostics::CrossFileDiagnosticKind::DestructuringBreaksReactivity { .. }
            )),
        "toRefs(inject(...)) should not be reported as reactivity loss"
    );
}

#[test]
fn test_playground_style_provide_inject() {
    // This test mimics the playground's exact setup (without template parsing)
    use crate::cross_file::diagnostics::CrossFileDiagnosticKind;

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

    let mut app_analyzer = crate::Analyzer::with_options(AnalyzerOptions::full());
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

    let mut parent_analyzer = crate::Analyzer::with_options(AnalyzerOptions::full());
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
    let mut child_analyzer = crate::Analyzer::with_options(AnalyzerOptions::full());
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
    use crate::cross_file::diagnostics::CrossFileDiagnosticKind;

    let mut analyzer =
        CrossFileAnalyzer::new(CrossFileOptions::default().with_provide_inject(true));

    // App.vue provides 'theme' and 'user'
    // App uses Child component in template (simulated via used_components)
    let mut app_analyzer = crate::Analyzer::with_options(AnalyzerOptions::full());
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
    let mut child_analyzer = crate::Analyzer::with_options(AnalyzerOptions::full());
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

#[test]
fn test_provide_inject_multiple_levels() {
    use crate::cross_file::diagnostics::CrossFileDiagnosticKind;

    let mut analyzer =
        CrossFileAnalyzer::new(CrossFileOptions::default().with_provide_inject(true));

    // Grandparent provides 'globalState'
    let mut gp_analyzer = crate::Analyzer::with_options(AnalyzerOptions::full());
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
    let mut parent_analyzer = crate::Analyzer::with_options(AnalyzerOptions::full());
    parent_analyzer.analyze_script_setup(r#"// No provide/inject"#);
    parent_analyzer
        .croquis_mut()
        .used_components
        .insert(vize_carton::CompactString::new("Child"));
    let parent_analysis = parent_analyzer.finish();

    // Child injects 'globalState' (from grandparent)
    let mut child_analyzer = crate::Analyzer::with_options(AnalyzerOptions::full());
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

    let mut app_analyzer = crate::Analyzer::with_options(AnalyzerOptions::full());
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

    let mut middle_analyzer = crate::Analyzer::with_options(AnalyzerOptions::full());
    middle_analyzer.analyze_script_setup("// pass-through component");
    middle_analyzer
        .croquis_mut()
        .used_components
        .insert(vize_carton::CompactString::new("Leaf"));
    let middle_analysis = middle_analyzer.finish();

    let mut leaf_analyzer = crate::Analyzer::with_options(AnalyzerOptions::full());
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
    use crate::cross_file::diagnostics::CrossFileDiagnosticKind;

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
    assert!(theme_matches
        .iter()
        .any(|provider_match| provider_match.provider == parent_a));
    assert!(theme_matches
        .iter()
        .any(|provider_match| provider_match.provider == parent_b));
    assert!(theme_matches
        .iter()
        .all(|provider_match| provider_match.consumer == child));
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

#[test]
fn test_nearest_ancestor_provider_shadows_grandparent() {
    use crate::cross_file::diagnostics::CrossFileDiagnosticKind;

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
    use crate::cross_file::diagnostics::CrossFileDiagnosticKind;

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
    use crate::cross_file::diagnostics::CrossFileDiagnosticKind;

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
    use crate::cross_file::diagnostics::CrossFileDiagnosticKind;
    use crate::cross_file::diagnostics::DiagnosticSeverity;

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

#[test]
fn test_shared_child_reactivity_loss_reports_each_provider_context() {
    use crate::cross_file::analyzers::CrossFileReactivityIssueKind;

    let mut analyzer =
        CrossFileAnalyzer::new(CrossFileOptions::default().with_reactivity_tracking(true));

    let parent_a = analyzer.add_file_with_analysis(
        Path::new("ParentA.vue"),
        "",
        script_analysis(
            r#"import { provide, reactive } from 'vue'
const state = reactive({ count: 1 })
provide('state', state)"#,
            &["Child"],
        ),
    );
    let parent_b = analyzer.add_file_with_analysis(
        Path::new("ParentB.vue"),
        "",
        script_analysis(
            r#"import { provide, reactive } from 'vue'
const state = reactive({ count: 2 })
provide('state', state)"#,
            &["Child"],
        ),
    );
    analyzer.add_file_with_analysis(
        Path::new("Child.vue"),
        "",
        script_analysis(
            r#"import { inject } from 'vue'
const { count } = inject('state') as { count: number }"#,
            &[],
        ),
    );
    analyzer.add_file_with_analysis(
        Path::new("App.vue"),
        "",
        script_analysis("// renders both provider branches", &["ParentA", "ParentB"]),
    );
    analyzer.rebuild_component_edges();

    let result = analyzer.analyze();
    let reactivity_losses = result
        .cross_file_reactivity_issues
        .iter()
        .filter(|issue| {
            matches!(
                &issue.kind,
                CrossFileReactivityIssueKind::InjectValueDestructured { key, .. } if key == "state"
            )
        })
        .collect::<Vec<_>>();

    assert_eq!(
        reactivity_losses.len(),
        2,
        "shared child destructuring should be tracked for each provider context"
    );
    assert!(reactivity_losses
        .iter()
        .any(|issue| issue.related_file == Some(parent_a)));
    assert!(reactivity_losses
        .iter()
        .any(|issue| issue.related_file == Some(parent_b)));
}

#[test]
fn test_reactivity_tracking_uses_later_same_component_provide() {
    use crate::cross_file::analyzers::CrossFileReactivityIssueKind;

    let mut analyzer =
        CrossFileAnalyzer::new(CrossFileOptions::default().with_reactivity_tracking(true));
    analyzer.add_file_with_analysis(
        Path::new("Parent.vue"),
        "",
        script_analysis(
            r#"import { provide, ref } from 'vue'
const staleState = { count: 0 }
const liveState = ref({ count: 1 })
provide('state', staleState)
provide('state', liveState)"#,
            &["Child"],
        ),
    );
    analyzer.add_file_with_analysis(
        Path::new("Child.vue"),
        "",
        script_analysis(
            r#"import { inject } from 'vue'
const state = inject('state')"#,
            &[],
        ),
    );
    analyzer.rebuild_component_edges();

    let result = analyzer.analyze();
    assert!(result.cross_file_reactivity_issues.iter().all(|issue| {
        !matches!(
            &issue.kind,
            CrossFileReactivityIssueKind::NonReactiveProvide { key } if key == "state"
        )
    }));
}

#[test]
fn test_provide_inject_does_not_match_string_and_symbol_keys() {
    use crate::cross_file::diagnostics::CrossFileDiagnosticKind;

    let mut analyzer =
        CrossFileAnalyzer::new(CrossFileOptions::default().with_provide_inject(true));

    let mut parent_analyzer = crate::Analyzer::with_options(AnalyzerOptions::full());
    parent_analyzer.analyze_script_setup(
        r#"import { provide, ref } from 'vue'
const ThemeKey = Symbol('theme')
const theme = ref('dark')
provide(ThemeKey, theme)"#,
    );
    parent_analyzer
        .croquis_mut()
        .used_components
        .insert(vize_carton::CompactString::new("Child"));
    let parent_analysis = parent_analyzer.finish();

    let mut child_analyzer = crate::Analyzer::with_options(AnalyzerOptions::full());
    child_analyzer.analyze_script_setup(
        r#"import { inject } from 'vue'
const theme = inject('ThemeKey')"#,
    );
    let child_analysis = child_analyzer.finish();

    analyzer.add_file_with_analysis(Path::new("Parent.vue"), "", parent_analysis);
    analyzer.add_file_with_analysis(Path::new("Child.vue"), "", child_analysis);
    analyzer.rebuild_component_edges();

    let result = analyzer.analyze();
    assert!(
        result.provide_inject_matches.is_empty(),
        "string keys must not match symbol keys with the same display text"
    );
    assert!(result.diagnostics.iter().any(|diagnostic| {
        matches!(
            &diagnostic.kind,
            CrossFileDiagnosticKind::UnmatchedInject { key } if key == "ThemeKey"
        )
    }));
}
