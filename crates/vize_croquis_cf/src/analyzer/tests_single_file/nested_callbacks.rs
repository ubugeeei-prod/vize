use super::{CrossFileAnalyzer, CrossFileOptions};
use vize_croquis::AnalyzerOptions;

#[test]
fn test_nested_callback_scopes() {
    let _analyzer = CrossFileAnalyzer::new(CrossFileOptions::minimal());

    // Use Analyzer directly for script setup context
    let mut single_analyzer = vize_croquis::Analyzer::with_options(AnalyzerOptions::full());
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
        .filter(|s| s.kind == vize_croquis::ScopeKind::Closure)
        .collect();

    assert!(
        closure_scopes.len() >= 3,
        "Should have at least 3 closure scopes (computed, map, filter)"
    );
}
