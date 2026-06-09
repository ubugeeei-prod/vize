use super::{CrossFileAnalyzer, CrossFileOptions, CrossFileResult};
use crate::CrossFileDiagnosticKind;
use std::path::Path;
use vize_carton::{CompactString, smallvec};
use vize_croquis::analysis::{ComponentUsage, EventListener, PassedProp};
use vize_croquis::{AnalyzerOptions, ScopeId};

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
fn test_component_resolution_reports_unregistered_pascal_case_component() {
    let mut analyzer = CrossFileAnalyzer::new(CrossFileOptions::strict());

    analyzer.add_file_with_analysis(
        Path::new("Parent.vue"),
        "",
        script_analysis_with_used_component("// No import of ChildWidget", "ChildWidget"),
    );

    let result = analyzer.analyze();

    assert_eq!(unregistered_components(&result), vec!["ChildWidget"]);
}

#[test]
fn test_component_resolution_ignores_custom_element_tag() {
    let mut analyzer = CrossFileAnalyzer::new(CrossFileOptions::strict());

    analyzer.add_file_with_analysis(
        Path::new("Parent.vue"),
        "",
        script_analysis_with_used_component("", "my-widget"),
    );

    let result = analyzer.analyze();

    assert!(unregistered_components(&result).is_empty());
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
fn test_undeclared_on_prefixed_prop_is_reported() {
    let mut analyzer =
        CrossFileAnalyzer::new(CrossFileOptions::default().with_props_validation(true));

    let child_analysis = script_analysis("const props = defineProps<{ title: string }>()");
    let parent_analysis = script_analysis_with_component_usage(
        r#"import Child from './Child.vue'"#,
        component_usage_with_on_prefixed_prop("Child"),
    );

    analyzer.add_file_with_analysis(Path::new("Child.vue"), "", child_analysis);
    analyzer.add_file_with_analysis(Path::new("Parent.vue"), "", parent_analysis);
    analyzer.rebuild_component_edges();

    let result = analyzer.analyze();
    let undeclared_props = result
        .diagnostics
        .iter()
        .filter_map(|diagnostic| match &diagnostic.kind {
            CrossFileDiagnosticKind::UndeclaredProp { prop_name, .. } => Some(prop_name.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>();

    assert_eq!(undeclared_props, vec!["online"]);
}

#[test]
fn test_spread_attrs_suppress_missing_required_prop_without_hiding_explicit_props() {
    let mut analyzer =
        CrossFileAnalyzer::new(CrossFileOptions::default().with_props_validation(true));

    let child_analysis = script_analysis("const props = defineProps<{ id: number }>()");
    let parent_analysis = script_analysis_with_component_usage(
        r#"import Child from './Child.vue'
const formData = { id: 1 }"#,
        component_usage_with_spread_and_extra_prop("Child"),
    );

    analyzer.add_file_with_analysis(Path::new("Child.vue"), "", child_analysis);
    analyzer.add_file_with_analysis(Path::new("Parent.vue"), "", parent_analysis);
    analyzer.rebuild_component_edges();

    let result = analyzer.analyze();
    let missing_required_props = result
        .diagnostics
        .iter()
        .filter_map(|diagnostic| match &diagnostic.kind {
            CrossFileDiagnosticKind::MissingRequiredProp { prop_name, .. } => {
                Some(prop_name.as_str())
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    let undeclared_props = result
        .diagnostics
        .iter()
        .filter_map(|diagnostic| match &diagnostic.kind {
            CrossFileDiagnosticKind::UndeclaredProp { prop_name, .. } => Some(prop_name.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>();

    assert!(missing_required_props.is_empty());
    assert_eq!(undeclared_props, vec!["extra"]);
}

fn script_analysis(script: &str) -> vize_croquis::Croquis {
    let mut analyzer = vize_croquis::Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.finish()
}

fn script_analysis_with_component_usage(
    script: &str,
    usage: ComponentUsage,
) -> vize_croquis::Croquis {
    let mut analysis = script_analysis(script);
    analysis.used_components.insert(usage.name.clone());
    analysis.component_usages.push(usage);
    analysis
}

fn script_analysis_with_used_component(script: &str, component: &str) -> vize_croquis::Croquis {
    let mut analysis = script_analysis(script);
    analysis
        .used_components
        .insert(CompactString::new(component));
    analysis
}

fn unregistered_components(result: &CrossFileResult) -> Vec<&str> {
    result
        .diagnostics
        .iter()
        .filter_map(|diagnostic| match &diagnostic.kind {
            CrossFileDiagnosticKind::UnregisteredComponent { component_name, .. } => {
                Some(component_name.as_str())
            }
            _ => None,
        })
        .collect()
}

fn component_usage_with_on_prefixed_prop(component: &str) -> ComponentUsage {
    ComponentUsage {
        name: CompactString::new(component),
        start: 0,
        end: 0,
        props: smallvec![
            passed_prop("title", Some("x"), false),
            passed_prop("online", Some("true"), false),
        ],
        events: smallvec![event_listener("click")],
        slots: smallvec![],
        has_spread_attrs: false,
        scope_id: ScopeId::ROOT,
        vif_guard: None,
    }
}

fn component_usage_with_spread_and_extra_prop(component: &str) -> ComponentUsage {
    ComponentUsage {
        name: CompactString::new(component),
        start: 0,
        end: 0,
        props: smallvec![passed_prop("extra", Some("true"), false)],
        events: smallvec![],
        slots: smallvec![],
        has_spread_attrs: true,
        scope_id: ScopeId::ROOT,
        vif_guard: None,
    }
}

fn passed_prop(name: &str, value: Option<&str>, is_dynamic: bool) -> PassedProp {
    PassedProp {
        name: CompactString::new(name),
        value: value.map(CompactString::new),
        start: 0,
        end: 0,
        is_dynamic,
    }
}

fn event_listener(name: &str) -> EventListener {
    EventListener {
        name: CompactString::new(name),
        handler: None,
        modifiers: smallvec![],
        start: 0,
        end: 0,
    }
}
