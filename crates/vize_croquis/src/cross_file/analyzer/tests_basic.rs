use super::{CrossFileAnalyzer, CrossFileOptions};
use std::path::Path;

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
