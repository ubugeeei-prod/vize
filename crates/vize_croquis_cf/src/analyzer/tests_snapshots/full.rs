use super::*;

#[test]
fn test_snapshot_full_cross_file_analysis() {
    let mut analyzer = CrossFileAnalyzer::new(CrossFileOptions::all());

    // App.vue - entry point with provide
    let mut app_analyzer = vize_croquis::Analyzer::with_options(AnalyzerOptions::full());
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
    let mut dashboard_analyzer = vize_croquis::Analyzer::with_options(AnalyzerOptions::full());
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
    let mut widget_analyzer = vize_croquis::Analyzer::with_options(AnalyzerOptions::full());
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
