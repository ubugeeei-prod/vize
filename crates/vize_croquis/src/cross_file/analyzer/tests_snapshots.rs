use super::{CrossFileAnalyzer, CrossFileOptions};
use crate::AnalyzerOptions;
use insta::assert_snapshot;
use std::path::Path;
use vize_carton::append;

#[test]
fn test_snapshot_full_cross_file_analysis() {
    let mut analyzer = CrossFileAnalyzer::new(CrossFileOptions::all());

    // App.vue - entry point with provide
    let mut app_analyzer = crate::Analyzer::with_options(AnalyzerOptions::full());
    app_analyzer.analyze_script_setup(
        r#"import { provide, ref, computed } from 'vue'

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
                append!(
                    output,
                    "      has_default: {}\n",
                    i.default_value.is_some()
                );
                append!(output, "      pattern: {:?}\n", i.pattern);
            }
        }
        output.push('\n');
    }

    assert_snapshot!(output);
}
