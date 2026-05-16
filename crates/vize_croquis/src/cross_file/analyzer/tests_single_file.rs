use super::{CrossFileAnalyzer, CrossFileOptions};
use crate::AnalyzerOptions;
use std::path::Path;

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
