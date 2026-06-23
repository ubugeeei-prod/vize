use super::{CrossFileAnalyzer, CrossFileOptions, CrossFileResult};
use crate::{CrossFileDiagnosticKind, PropsValidationIssueKind};
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

#[test]
fn test_props_validation_normalizes_kebab_case_prop_bindings() {
    for (case, value, is_dynamic) in [
        ("static attr", Some("x"), false),
        ("dynamic bind", Some("'x'"), true),
    ] {
        let mut analyzer =
            CrossFileAnalyzer::new(CrossFileOptions::default().with_props_validation(true));

        let child_analysis = script_analysis("const props = defineProps<{ userName: string }>()");
        let parent_analysis = script_analysis_with_component_usage(
            r#"import Child from './Child.vue'"#,
            component_usage_with_prop("Child", "user-name", value, is_dynamic),
        );

        analyzer.add_file_with_analysis(Path::new("Child.vue"), "", child_analysis);
        analyzer.add_file_with_analysis(Path::new("Parent.vue"), "", parent_analysis);
        analyzer.rebuild_component_edges();

        let result = analyzer.analyze();
        let prop_validation_diagnostics = result
            .diagnostics
            .iter()
            .filter(|diagnostic| {
                matches!(
                    diagnostic.kind,
                    CrossFileDiagnosticKind::MissingRequiredProp { .. }
                        | CrossFileDiagnosticKind::UndeclaredProp { .. }
                )
            })
            .collect::<Vec<_>>();

        assert!(
            prop_validation_diagnostics.is_empty(),
            "{case} should match camelCase defineProps without missing/undeclared diagnostics"
        );
    }
}

#[test]
fn test_props_validation_uses_component_and_prop_offsets() {
    let mut analyzer =
        CrossFileAnalyzer::new(CrossFileOptions::default().with_props_validation(true));

    let child_analysis = script_analysis("const props = defineProps<{ id: number }>()");
    let parent_analysis = script_analysis_with_component_usage(
        r#"import Child from './Child.vue'"#,
        component_usage_with_extra_prop_at("Child", 80, 89, 96),
    );

    analyzer.add_file_with_analysis(Path::new("Child.vue"), "", child_analysis);
    analyzer.add_file_with_analysis(Path::new("Parent.vue"), "", parent_analysis);
    analyzer.rebuild_component_edges();

    let result = analyzer.analyze();

    let missing_required = result
        .diagnostics
        .iter()
        .find(|diagnostic| {
            matches!(
                diagnostic.kind,
                CrossFileDiagnosticKind::MissingRequiredProp { .. }
            )
        })
        .expect("missing required prop diagnostic");
    assert_eq!(missing_required.primary_offset, 80);
    assert_eq!(missing_required.primary_end_offset, 96);
    assert_eq!(missing_required.related_files.len(), 1);
    assert!(
        missing_required.related_files[0].1 > 0,
        "related defineProps offset should not be hardcoded to zero"
    );

    let undeclared = result
        .diagnostics
        .iter()
        .find(|diagnostic| {
            matches!(
                diagnostic.kind,
                CrossFileDiagnosticKind::UndeclaredProp { .. }
            )
        })
        .expect("undeclared prop diagnostic");
    assert_eq!(undeclared.primary_offset, 89);
    assert_eq!(undeclared.primary_end_offset, 96);
}

#[test]
fn test_props_validation_emits_type_mismatch_for_static_literal() {
    let mut analyzer =
        CrossFileAnalyzer::new(CrossFileOptions::default().with_props_validation(true));

    let child_analysis =
        script_analysis("const props = defineProps({ count: { type: Number, required: true } })");
    let parent_analysis = script_analysis_with_component_usage(
        r#"import Child from './Child.vue'"#,
        component_usage_with_count_string_at("Child", 20, 27, 42),
    );

    analyzer.add_file_with_analysis(Path::new("Child.vue"), "", child_analysis);
    analyzer.add_file_with_analysis(Path::new("Parent.vue"), "", parent_analysis);
    analyzer.rebuild_component_edges();

    let result = analyzer.analyze();

    let issue = result
        .props_validation_issues
        .iter()
        .find_map(|issue| match &issue.kind {
            PropsValidationIssueKind::TypeMismatch {
                prop_name,
                expected,
                actual,
            } => Some((
                prop_name.as_str(),
                expected.as_str(),
                actual.as_str(),
                issue.offset,
            )),
            _ => None,
        })
        .expect("type mismatch issue");
    assert_eq!(issue, ("count", "number", "string", 27));

    let diagnostic = result
        .diagnostics
        .iter()
        .find_map(|diagnostic| match &diagnostic.kind {
            CrossFileDiagnosticKind::PropTypeMismatch {
                prop_name,
                expected_type,
                actual_type,
            } => Some((
                prop_name.as_str(),
                expected_type.as_str(),
                actual_type.as_str(),
                diagnostic.primary_offset,
                diagnostic.primary_end_offset,
            )),
            _ => None,
        })
        .expect("type mismatch diagnostic");

    assert_eq!(diagnostic, ("count", "number", "string", 27, 42));
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

fn component_usage_with_extra_prop_at(
    component: &str,
    usage_start: u32,
    prop_start: u32,
    prop_end: u32,
) -> ComponentUsage {
    ComponentUsage {
        name: CompactString::new(component),
        start: usage_start,
        end: prop_end,
        props: smallvec![passed_prop_at(
            "extra",
            Some("true"),
            false,
            prop_start,
            prop_end
        )],
        events: smallvec![],
        slots: smallvec![],
        has_spread_attrs: false,
        scope_id: ScopeId::ROOT,
        vif_guard: None,
    }
}

fn component_usage_with_count_string_at(
    component: &str,
    usage_start: u32,
    prop_start: u32,
    prop_end: u32,
) -> ComponentUsage {
    ComponentUsage {
        name: CompactString::new(component),
        start: usage_start,
        end: prop_end,
        props: smallvec![passed_prop_at(
            "count",
            Some("'not a number'"),
            true,
            prop_start,
            prop_end,
        )],
        events: smallvec![],
        slots: smallvec![],
        has_spread_attrs: false,
        scope_id: ScopeId::ROOT,
        vif_guard: None,
    }
}

fn component_usage_with_prop(
    component: &str,
    prop_name: &str,
    value: Option<&str>,
    is_dynamic: bool,
) -> ComponentUsage {
    ComponentUsage {
        name: CompactString::new(component),
        start: 0,
        end: 0,
        props: smallvec![passed_prop(prop_name, value, is_dynamic)],
        events: smallvec![],
        slots: smallvec![],
        has_spread_attrs: false,
        scope_id: ScopeId::ROOT,
        vif_guard: None,
    }
}

fn passed_prop(name: &str, value: Option<&str>, is_dynamic: bool) -> PassedProp {
    passed_prop_at(name, value, is_dynamic, 0, 0)
}

fn passed_prop_at(
    name: &str,
    value: Option<&str>,
    is_dynamic: bool,
    start: u32,
    end: u32,
) -> PassedProp {
    PassedProp {
        name: CompactString::new(name),
        value: value.map(CompactString::new),
        start,
        end,
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
