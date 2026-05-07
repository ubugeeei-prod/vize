//! Main cross-file analyzer.
//!
//! Orchestrates all cross-file analysis passes and manages the module registry
//! and dependency graph.

mod core;
mod types;

pub use core::CrossFileAnalyzer;
pub use types::{CrossFileOptions, CrossFileResult, CrossFileStats};

#[cfg(test)]
mod tests {
    use super::{CrossFileAnalyzer, CrossFileOptions};
    use crate::analysis::ComponentUsage;
    use crate::cross_file::diagnostics::CrossFileDiagnosticKind;
    use crate::cross_file::DependencyEdge;
    use crate::AnalyzerOptions;
    use std::path::Path;
    use vize_carton::append;
    use vize_carton::{CompactString, SmallVec};

    #[test]
    fn test_cross_file_options() {
        let options = CrossFileOptions::default();
        assert!(!options.any_enabled());

        let options = CrossFileOptions::all();
        assert!(options.any_enabled());
        assert!(options.fallthrough_attrs);
        assert!(options.reactivity_tracking);
        assert!(options.component_resolution);
        assert!(options.props_validation);
    }

    #[test]
    fn test_strict_options() {
        let options = CrossFileOptions::strict();
        assert!(options.component_resolution);
        assert!(options.props_validation);
        assert!(options.circular_dependencies);
        // Other options should be disabled
        assert!(!options.fallthrough_attrs);
        assert!(!options.event_bubbling);
    }

    #[test]
    fn test_analyzer_basic() {
        let mut analyzer = CrossFileAnalyzer::new(CrossFileOptions::minimal());

        let id = analyzer.add_file(
            Path::new("Test.vue"),
            "<script setup>\nconst count = ref(0)\n</script>",
        );

        assert_eq!(analyzer.registry().len(), 1);
        assert!(analyzer.get_analysis(id).is_some());
    }

    #[test]
    fn test_component_resolution_error() {
        let mut analyzer = CrossFileAnalyzer::new(CrossFileOptions::strict());

        // Add a file that uses an unregistered component
        analyzer.add_file(
            Path::new("Parent.vue"),
            r#"<script setup>
// No import of ChildComponent
</script>"#,
        );

        // When template analysis is added, this test will verify
        // that unregistered components produce errors
    }

    #[test]
    fn test_circular_dependency_detection() {
        let mut analyzer = CrossFileAnalyzer::new(CrossFileOptions::strict());

        // This test would require adding files with circular imports
        // For now, just verify the analysis runs without crashing
        let result = analyzer.analyze();
        assert!(result.circular_deps.is_empty());
    }

    #[test]
    fn test_rebuild_import_edges_resolves_relative_imports_added_out_of_order() {
        let mut analyzer = CrossFileAnalyzer::new(CrossFileOptions::strict());

        let mut parent_analyzer = crate::Analyzer::with_options(AnalyzerOptions::full());
        parent_analyzer.analyze_script_setup("import Child from './components/Child.vue'");
        let parent_id = analyzer.add_file_with_analysis(
            Path::new("src/Parent.vue"),
            "",
            parent_analyzer.finish(),
        );

        let mut child_analyzer = crate::Analyzer::with_options(AnalyzerOptions::full());
        child_analyzer.analyze_script_setup("const child = true");
        let child_id = analyzer.add_file_with_analysis(
            Path::new("src/components/Child.vue"),
            "",
            child_analyzer.finish(),
        );

        assert!(!analyzer
            .graph()
            .dependencies(parent_id)
            .any(|(id, edge)| id == child_id && edge == DependencyEdge::Import));

        analyzer.rebuild_import_edges();

        assert!(analyzer
            .graph()
            .dependencies(parent_id)
            .any(|(id, edge)| id == child_id && edge == DependencyEdge::Import));
    }

    #[test]
    fn test_import_and_component_usage_edges_can_share_same_target() {
        let mut analyzer = CrossFileAnalyzer::new(CrossFileOptions::strict());

        let mut parent_analyzer = crate::Analyzer::with_options(AnalyzerOptions::full());
        parent_analyzer.analyze_script_setup("import Widget from './Child.vue'");
        parent_analyzer
            .croquis_mut()
            .used_components
            .insert(CompactString::new("Widget"));
        parent_analyzer
            .croquis_mut()
            .component_usages
            .push(ComponentUsage {
                name: CompactString::new("Widget"),
                start: 0,
                end: 8,
                props: SmallVec::new(),
                events: SmallVec::new(),
                slots: SmallVec::new(),
                has_spread_attrs: false,
                scope_id: crate::scope::ScopeId::ROOT,
                vif_guard: None,
            });
        let parent_id =
            analyzer.add_file_with_analysis(Path::new("Parent.vue"), "", parent_analyzer.finish());

        let mut child_analyzer = crate::Analyzer::with_options(AnalyzerOptions::full());
        child_analyzer.analyze_script_setup(
            r#"defineProps<{
  title: string
}>()"#,
        );
        let child_id =
            analyzer.add_file_with_analysis(Path::new("Child.vue"), "", child_analyzer.finish());

        analyzer.rebuild_import_edges();
        analyzer.rebuild_component_edges();

        let edges: Vec<_> = analyzer.graph().dependencies(parent_id).collect();
        assert!(edges
            .iter()
            .any(|(id, edge)| *id == child_id && *edge == DependencyEdge::Import));
        assert!(edges
            .iter()
            .any(|(id, edge)| *id == child_id && *edge == DependencyEdge::ComponentUsage));

        let result = analyzer.analyze();
        assert!(result.diagnostics.iter().any(|diagnostic| matches!(
            diagnostic.kind,
            CrossFileDiagnosticKind::MissingRequiredProp { .. }
        )));
    }

    #[test]
    fn test_component_resolution_requires_local_registration() {
        let mut analyzer = CrossFileAnalyzer::new(CrossFileOptions::strict());

        let mut parent_analyzer = crate::Analyzer::with_options(AnalyzerOptions::full());
        parent_analyzer
            .croquis_mut()
            .used_components
            .insert(CompactString::new("Child"));
        analyzer.add_file_with_analysis(Path::new("Parent.vue"), "", parent_analyzer.finish());

        let mut child_analyzer = crate::Analyzer::with_options(AnalyzerOptions::full());
        child_analyzer.analyze_script_setup("const child = true");
        analyzer.add_file_with_analysis(Path::new("Child.vue"), "", child_analyzer.finish());
        analyzer.rebuild_component_edges();

        let result = analyzer.analyze();
        assert!(result.diagnostics.iter().any(|diagnostic| matches!(
            diagnostic.kind,
            CrossFileDiagnosticKind::UnregisteredComponent { .. }
        )));
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
    fn test_reactivity_wrappers_detected() {
        let mut analyzer = CrossFileAnalyzer::new(CrossFileOptions::minimal());

        analyzer.add_file(
            Path::new("Test.ts"),
            r#"import { ref, computed, reactive, shallowRef, toRef, toRefs } from 'vue'

const count = ref(0)
const doubled = computed(() => count.value * 2)
const state = reactive({ name: 'test' })
const shallow = shallowRef({ deep: 'value' })
const props = defineProps<{ item: { name: string } }>()
const nameRef = toRef(props, 'item')"#,
        );

        let analysis = analyzer
            .get_analysis(analyzer.registry().iter().next().unwrap().id)
            .unwrap();

        // Check reactivity tracking
        assert!(analysis.reactivity.is_reactive("count"));
        assert!(analysis.reactivity.is_reactive("doubled"));
        assert!(analysis.reactivity.is_reactive("state"));
        assert!(analysis.reactivity.is_reactive("shallow"));
        assert!(analysis.reactivity.is_reactive("nameRef"));
    }

    #[test]
    fn test_define_props_with_type() {
        let mut analyzer = CrossFileAnalyzer::new(CrossFileOptions::minimal());

        analyzer.add_file(
            Path::new("Test.ts"),
            r#"const props = defineProps<{
    msg: string
    count?: number
    user: { name: string; age: number }
}>()"#,
        );

        let analysis = analyzer
            .get_analysis(analyzer.registry().iter().next().unwrap().id)
            .unwrap();

        assert_eq!(analysis.macros.props().len(), 3);
        assert!(analysis
            .macros
            .props()
            .iter()
            .any(|p| p.name.as_str() == "msg" && p.required));
        assert!(analysis
            .macros
            .props()
            .iter()
            .any(|p| p.name.as_str() == "count" && !p.required));
        assert!(analysis
            .macros
            .props()
            .iter()
            .any(|p| p.name.as_str() == "user" && p.required));
    }

    #[test]
    fn test_define_emits() {
        let mut analyzer = CrossFileAnalyzer::new(CrossFileOptions::minimal());

        analyzer.add_file(
            Path::new("Test.ts"),
            r#"const emit = defineEmits<{
    (e: 'update', value: string): void
    (e: 'delete', id: number): void
}>()"#,
        );

        let analysis = analyzer
            .get_analysis(analyzer.registry().iter().next().unwrap().id)
            .unwrap();

        assert_eq!(analysis.macros.emits().len(), 2);
    }

    #[test]
    fn test_invalid_exports_in_script_setup() {
        let _analyzer = CrossFileAnalyzer::new(CrossFileOptions::minimal());

        // Use Analyzer directly for script setup context
        let mut single_analyzer = crate::Analyzer::with_options(AnalyzerOptions::full());
        single_analyzer.analyze_script_setup(
            r#"export const foo = 'bar'
export function hello() {}
export default {}"#,
        );
        let analysis = single_analyzer.finish();

        // Should detect invalid exports in script setup
        assert!(analysis.invalid_exports.len() >= 2);
    }

    #[test]
    fn test_type_exports_allowed() {
        let _analyzer = CrossFileAnalyzer::new(CrossFileOptions::minimal());

        // Use Analyzer directly for script setup context
        let mut single_analyzer = crate::Analyzer::with_options(AnalyzerOptions::full());
        single_analyzer.analyze_script_setup(
            r#"export type Props = { msg: string }
export interface Emits {
    (e: 'update', value: string): void
}"#,
        );
        let analysis = single_analyzer.finish();

        // Type exports should be allowed and tracked
        assert_eq!(analysis.type_exports.len(), 2);
        // No invalid exports for type declarations
        assert_eq!(analysis.invalid_exports.len(), 0);
    }

    #[test]
    fn test_scope_tracking_lifecycle_hooks() {
        let _analyzer = CrossFileAnalyzer::new(CrossFileOptions::minimal());

        // Use Analyzer directly for script setup context
        let mut single_analyzer = crate::Analyzer::with_options(AnalyzerOptions::full());
        single_analyzer.analyze_script_setup(
            r#"import { onMounted, onUnmounted, ref } from 'vue'

const count = ref(0)

onMounted(() => {
    console.log('mounted')
    count.value++
})

onUnmounted(() => {
    console.log('unmounted')
})"#,
        );
        let analysis = single_analyzer.finish();

        // Should have scopes for lifecycle hooks (client-only scopes)
        let client_only_scopes: Vec<_> = analysis
            .scopes
            .iter()
            .filter(|s| s.kind == crate::scope::ScopeKind::ClientOnly)
            .collect();

        assert_eq!(
            client_only_scopes.len(),
            2,
            "Should have 2 client-only scopes for onMounted and onUnmounted"
        );
    }

    #[test]
    fn test_nested_callback_scopes() {
        let _analyzer = CrossFileAnalyzer::new(CrossFileOptions::minimal());

        // Use Analyzer directly for script setup context
        let mut single_analyzer = crate::Analyzer::with_options(AnalyzerOptions::full());
        single_analyzer.analyze_script_setup(
            r#"import { computed } from 'vue'

const items = computed(() => {
    return list.map(item => {
        return item.value.filter(v => v > 0)
    })
})"#,
        );
        let analysis = single_analyzer.finish();

        // Should have multiple closure scopes for nested callbacks
        let closure_scopes: Vec<_> = analysis
            .scopes
            .iter()
            .filter(|s| s.kind == crate::scope::ScopeKind::Closure)
            .collect();

        assert!(
            closure_scopes.len() >= 3,
            "Should have at least 3 closure scopes (computed, map, filter)"
        );
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
        let _child_id = analyzer.add_file_with_analysis(
            Path::new("Child.vue"),
            "script content",
            child_analysis,
        );

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

    // === Snapshot Tests ===

    #[test]
    fn test_snapshot_full_cross_file_analysis() {
        use insta::assert_snapshot;

        let mut analyzer = CrossFileAnalyzer::new(CrossFileOptions::all());

        // App.vue - entry point with provide
        let mut app_analyzer = crate::Analyzer::with_options(AnalyzerOptions::full());
        app_analyzer.analyze_script_setup(
            r#"import { provide, ref, computed } from 'vue'
import Dashboard from './Dashboard.vue'

const theme = ref('dark')
const user = ref({ name: 'Alice', role: 'admin' })

provide('theme', theme)
provide('user', user)

const isAdmin = computed(() => user.value.role === 'admin')"#,
        );
        app_analyzer
            .croquis_mut()
            .used_components
            .insert(vize_carton::CompactString::new("Dashboard"));
        let app_analysis = app_analyzer.finish();

        // Dashboard.vue - uses theme, provides nested state
        let mut dashboard_analyzer = crate::Analyzer::with_options(AnalyzerOptions::full());
        dashboard_analyzer.analyze_script_setup(
            r#"import { inject, provide, ref } from 'vue'
import Widget from './Widget.vue'

const theme = inject('theme')
const user = inject('user')
const dashboardState = ref({ count: 0 })

provide('dashboardState', dashboardState)"#,
        );
        dashboard_analyzer
            .croquis_mut()
            .used_components
            .insert(vize_carton::CompactString::new("Widget"));
        let dashboard_analysis = dashboard_analyzer.finish();

        // Widget.vue - uses all injected values
        let mut widget_analyzer = crate::Analyzer::with_options(AnalyzerOptions::full());
        widget_analyzer.analyze_script_setup(
            r#"import { inject, computed } from 'vue'

const theme = inject('theme')
const dashboardState = inject('dashboardState')
const displayCount = computed(() => dashboardState.value.count)"#,
        );
        let widget_analysis = widget_analyzer.finish();

        // Add files
        analyzer.add_file_with_analysis(Path::new("App.vue"), "", app_analysis);
        analyzer.add_file_with_analysis(Path::new("Dashboard.vue"), "", dashboard_analysis);
        analyzer.add_file_with_analysis(Path::new("Widget.vue"), "", widget_analysis);

        analyzer.rebuild_import_edges();
        analyzer.rebuild_component_edges();
        let result = analyzer.analyze();

        // Build snapshot output
        let mut output = String::new();

        output.push_str("=== Cross-File Analysis Result ===\n\n");

        output.push_str("== Statistics ==\n");
        append!(output, "Files analyzed: {}\n", result.stats.files_analyzed);
        append!(output, "Vue components: {}\n", result.stats.vue_components);
        append!(
            output,
            "Dependency edges: {}\n",
            result.stats.dependency_edges
        );
        append!(output, "Errors: {}\n", result.stats.error_count);
        append!(output, "Warnings: {}\n", result.stats.warning_count);

        output.push_str("\n== Provide/Inject Matches ==\n");
        for m in &result.provide_inject_matches {
            append!(output, "  {:?} -> {:?}\n", m.provider, m.consumer);
            append!(output, "    key: {:?}\n", m.key);
        }

        output.push_str("\n== Diagnostics ==\n");
        // Sort diagnostics for deterministic output
        let mut sorted_diags = result.diagnostics.clone();
        sorted_diags.sort_by(|a, b| a.message.cmp(&b.message));
        for d in &sorted_diags {
            append!(
                output,
                "  [{}] {:?}: {}\n",
                if d.is_error() {
                    "ERROR"
                } else if d.is_warning() {
                    "WARN"
                } else {
                    "INFO"
                },
                d.primary_file,
                d.message
            );
        }

        assert_snapshot!(output);
    }

    #[test]
    fn test_snapshot_dependency_graph() {
        use insta::assert_snapshot;

        let mut analyzer =
            CrossFileAnalyzer::new(CrossFileOptions::default().with_provide_inject(true));

        // Create a complex dependency graph
        let mut comp_a = crate::Analyzer::with_options(AnalyzerOptions::full());
        comp_a.analyze_script_setup(
            r#"import { provide } from 'vue'
provide('a', 1)"#,
        );
        comp_a
            .croquis_mut()
            .used_components
            .insert(vize_carton::CompactString::new("CompB"));
        comp_a
            .croquis_mut()
            .used_components
            .insert(vize_carton::CompactString::new("CompC"));

        let mut comp_b = crate::Analyzer::with_options(AnalyzerOptions::full());
        comp_b.analyze_script_setup(
            r#"import { inject, provide } from 'vue'
const a = inject('a')
provide('b', 2)"#,
        );
        comp_b
            .croquis_mut()
            .used_components
            .insert(vize_carton::CompactString::new("CompD"));

        let mut comp_c = crate::Analyzer::with_options(AnalyzerOptions::full());
        comp_c.analyze_script_setup(
            r#"import { inject } from 'vue'
const a = inject('a')"#,
        );
        comp_c
            .croquis_mut()
            .used_components
            .insert(vize_carton::CompactString::new("CompD"));

        let mut comp_d = crate::Analyzer::with_options(AnalyzerOptions::full());
        comp_d.analyze_script_setup(
            r#"import { inject } from 'vue'
const b = inject('b')"#,
        );

        analyzer.add_file_with_analysis(Path::new("CompA.vue"), "", comp_a.finish());
        analyzer.add_file_with_analysis(Path::new("CompB.vue"), "", comp_b.finish());
        analyzer.add_file_with_analysis(Path::new("CompC.vue"), "", comp_c.finish());
        analyzer.add_file_with_analysis(Path::new("CompD.vue"), "", comp_d.finish());

        analyzer.rebuild_component_edges();

        // Build graph output
        let mut output = String::new();
        output.push_str("=== Dependency Graph ===\n\n");

        let mut nodes: Vec<_> = analyzer.graph().nodes().collect();
        nodes.sort_by(|a, b| a.path.cmp(&b.path));

        for node in nodes {
            append!(output, "Node: {}\n", node.path);
            append!(output, "  component_name: {:?}\n", node.component_name);
            append!(output, "  is_entry: {}\n", node.is_entry);
            append!(output, "  imports: {:?}\n", node.imports);
            output.push('\n');
        }

        assert_snapshot!(output);
    }

    #[test]
    fn test_snapshot_reactivity_issues() {
        use insta::assert_snapshot;

        let mut analyzer =
            CrossFileAnalyzer::new(CrossFileOptions::default().with_reactivity_tracking(true));

        // File with reactivity issues
        let mut comp = crate::Analyzer::with_options(AnalyzerOptions::full());
        comp.analyze_script_setup(
            r#"import { inject, ref, computed } from 'vue'

// Good: Simple inject
const theme = inject('theme')

// Issue: Destructuring inject loses reactivity
const { count, name } = inject('state') as { count: number; name: string }

// Good: Using computed
const doubled = computed(() => count * 2)

// Good: Using ref
const localCount = ref(0)"#,
        );
        let analysis = comp.finish();

        analyzer.add_file_with_analysis(Path::new("Component.vue"), "", analysis);
        let result = analyzer.analyze();

        // Build output
        let mut output = String::new();
        output.push_str("=== Reactivity Analysis ===\n\n");

        output.push_str("== Reactivity Issues ==\n");
        for issue in &result.reactivity_issues {
            append!(output, "  File: {:?}\n", issue.file_id);
            append!(output, "    kind: {:?}\n", issue.kind);
            append!(output, "    source: {:?}\n", issue.source);
            output.push('\n');
        }

        output.push_str("== Cross-File Reactivity Issues ==\n");
        for issue in &result.cross_file_reactivity_issues {
            append!(output, "  File: {:?}\n", issue.file_id);
            append!(output, "    kind: {:?}\n", issue.kind);
            append!(output, "    related_file: {:?}\n", issue.related_file);
            output.push('\n');
        }

        assert_snapshot!(output);
    }

    #[test]
    fn test_snapshot_provide_inject_patterns() {
        use insta::assert_snapshot;

        let mut analyzer =
            CrossFileAnalyzer::new(CrossFileOptions::default().with_provide_inject(true));

        // Various provide patterns
        let mut provider = crate::Analyzer::with_options(AnalyzerOptions::full());
        provider.analyze_script_setup(
            r#"import { provide, ref, reactive, computed } from 'vue'

// String key provides
provide('stringKey', 'value')
provide('refValue', ref(0))
provide('reactiveValue', reactive({ a: 1 }))
provide('computedValue', computed(() => 42))

// Symbol key provide
const ThemeSymbol = Symbol('theme')
provide(ThemeSymbol, 'dark')"#,
        );

        // Various inject patterns
        let mut consumer = crate::Analyzer::with_options(AnalyzerOptions::full());
        consumer.analyze_script_setup(
            r#"import { inject } from 'vue'

// Simple inject
const str = inject('stringKey')

// Inject with default
const withDefault = inject('missing', 'default')

// Inject with type assertion
const typed = inject('refValue') as Ref<number>

// Destructuring inject
const { a } = inject('reactiveValue') as { a: number }

// Inject computed
const comp = inject('computedValue')"#,
        );

        analyzer.add_file_with_analysis(Path::new("Provider.vue"), "", provider.finish());
        analyzer.add_file_with_analysis(Path::new("Consumer.vue"), "", consumer.finish());

        // Build output
        let mut output = String::new();
        output.push_str("=== Provide/Inject Patterns ===\n\n");

        for entry in analyzer.registry().iter() {
            append!(output, "File: {}\n", entry.filename);

            if !entry.analysis.provide_inject.provides().is_empty() {
                output.push_str("  Provides:\n");
                for p in entry.analysis.provide_inject.provides() {
                    append!(output, "    - key: {:?}\n", p.key);
                    append!(output, "      value: {}\n", p.value);
                }
            }

            if !entry.analysis.provide_inject.injects().is_empty() {
                output.push_str("  Injects:\n");
                for i in entry.analysis.provide_inject.injects() {
                    append!(output, "    - key: {:?}\n", i.key);
                    append!(output, "      has_default: {}\n", i.default_value.is_some());
                    append!(output, "      pattern: {:?}\n", i.pattern);
                }
            }
            output.push('\n');
        }

        assert_snapshot!(output);
    }
}
