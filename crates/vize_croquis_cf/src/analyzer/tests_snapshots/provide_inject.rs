use super::*;
use vize_carton::CompactString;

#[test]
fn test_snapshot_provide_inject_tree_outputs() {
    let mut analyzer =
        CrossFileAnalyzer::new(CrossFileOptions::default().with_provide_inject(true));

    analyzer.add_file_with_analysis(
        Path::new("App.vue"),
        "",
        script_analysis(
            r#"import { provide, ref } from 'vue'
const theme = ref('dark')
provide('theme', theme)"#,
            &["Layout"],
        ),
    );
    analyzer.add_file_with_analysis(
        Path::new("Layout.vue"),
        "",
        script_analysis("// pass-through layout", &["Panel", "Sidebar"]),
    );
    analyzer.add_file_with_analysis(
        Path::new("Panel.vue"),
        "",
        script_analysis(
            r#"import { inject, provide, ref } from 'vue'
const theme = inject('theme')
const panelState = ref({ open: true })
provide('panelState', panelState)"#,
            &["Button"],
        ),
    );
    analyzer.add_file_with_analysis(
        Path::new("Button.vue"),
        "",
        script_analysis(
            r#"import { inject } from 'vue'
const panelState = inject('panelState')"#,
            &[],
        ),
    );
    analyzer.add_file_with_analysis(
        Path::new("Sidebar.vue"),
        "",
        script_analysis(
            r#"import { inject } from 'vue'
const theme = inject('theme')"#,
            &[],
        ),
    );
    analyzer.add_file_with_analysis(
        Path::new("Orphan.vue"),
        "",
        script_analysis(
            r#"import { inject } from 'vue'
const missing = inject('missing', 'fallback')"#,
            &[],
        ),
    );
    analyzer.add_file_with_analysis(
        Path::new("UnusedProvider.vue"),
        "",
        script_analysis(
            r#"import { provide } from 'vue'
provide('stale', 1)"#,
            &[],
        ),
    );
    analyzer.rebuild_component_edges();

    let result = analyzer.analyze();
    let tree = result
        .provide_inject_tree
        .as_ref()
        .expect("tree should be built");
    let summary = result
        .provide_inject_tree_summary
        .expect("tree summary should be built");

    assert_eq!(summary.root_count, 3);
    assert_eq!(summary.node_count, 7);
    assert_eq!(summary.pass_through_component_count, 1);
    assert_eq!(summary.provider_component_count, 3);
    assert_eq!(summary.injector_component_count, 4);
    assert_eq!(summary.matched_inject_count, 3);
    assert_eq!(summary.unmatched_inject_count, 1);
    assert_eq!(summary.max_depth, 4);
    assert_eq!(summary.max_child_fanout, 2);
    assert_eq!(summary.max_provider_consumer_count, 2);

    let mut output = String::new();
    output.push_str("=== Provide/Inject Tree Outputs ===\n\n");
    output.push_str("== Summary ==\n");
    output.push_str(
        serde_json::to_string_pretty(&summary)
            .expect("summary should serialize")
            .as_str(),
    );
    output.push_str("\n\n== Tree JSON ==\n");
    output.push_str(
        serde_json::to_string_pretty(tree)
            .expect("tree should serialize")
            .as_str(),
    );
    output.push_str("\n\n== Markdown ==\n");
    output.push_str(tree.to_markdown(analyzer.registry()).as_str());

    assert_snapshot!(output);
}

#[test]
fn test_snapshot_shared_child_branch_provider_context() {
    let mut analyzer =
        CrossFileAnalyzer::new(CrossFileOptions::default().with_provide_inject(true));

    // Deliberately register out of display order so the snapshot also guards
    // against FileId-based ordering.
    analyzer.add_file_with_analysis(
        Path::new("Child.vue"),
        "",
        script_analysis(
            r#"import { inject } from 'vue'
const theme = inject('theme')"#,
            &[],
        ),
    );
    analyzer.add_file_with_analysis(
        Path::new("ParentB.vue"),
        "",
        script_analysis(
            r#"import { provide } from 'vue'
provide('theme', 'light')"#,
            &["Child"],
        ),
    );
    analyzer.add_file_with_analysis(
        Path::new("App.vue"),
        "",
        script_analysis("// renders both providers", &["ParentA", "ParentB"]),
    );
    analyzer.add_file_with_analysis(
        Path::new("ParentA.vue"),
        "",
        script_analysis(
            r#"import { provide } from 'vue'
provide('theme', 'dark')"#,
            &["Child"],
        ),
    );
    analyzer.rebuild_component_edges();

    let result = analyzer.analyze();
    let tree = result
        .provide_inject_tree
        .as_ref()
        .expect("tree should be built");
    let summary = result
        .provide_inject_tree_summary
        .expect("tree summary should be built");

    let mut output = String::new();
    output.push_str("=== Shared Child Branch Context ===\n\n");
    output.push_str("== Summary ==\n");
    output.push_str(
        serde_json::to_string_pretty(&summary)
            .expect("summary should serialize")
            .as_str(),
    );
    output.push_str("\n\n== Tree JSON ==\n");
    output.push_str(
        serde_json::to_string_pretty(tree)
            .expect("tree should serialize")
            .as_str(),
    );
    output.push_str("\n\n== Markdown ==\n");
    output.push_str(tree.to_markdown(analyzer.registry()).as_str());

    assert_snapshot!(output);
}

#[test]
fn test_snapshot_partial_shared_child_branches() {
    let mut analyzer =
        CrossFileAnalyzer::new(CrossFileOptions::default().with_provide_inject(true));
    analyzer.add_file_with_analysis(
        Path::new("Child.vue"),
        "",
        script_analysis(
            "import { inject } from 'vue'; const theme = inject('theme'); const locale = inject('locale', 'en')",
            &[],
        ),
    );
    analyzer.add_file_with_analysis(
        Path::new("ParentB.vue"),
        "",
        script_analysis(
            "import { provide } from 'vue'; provide('locale', 'ja')",
            &["Child"],
        ),
    );
    analyzer.add_file_with_analysis(
        Path::new("App.vue"),
        "",
        script_analysis("// renders disjoint providers", &["ParentA", "ParentB"]),
    );
    analyzer.add_file_with_analysis(
        Path::new("ParentA.vue"),
        "",
        script_analysis(
            "import { provide } from 'vue'; provide('theme', 'dark')",
            &["Child"],
        ),
    );
    analyzer.rebuild_component_edges();

    let result = analyzer.analyze();
    let tree = result
        .provide_inject_tree
        .as_ref()
        .expect("tree should be built");
    let summary = result
        .provide_inject_tree_summary
        .expect("tree summary should be built");

    let mut output = String::new();
    output.push_str("=== Partial Shared Child Branches ===\n\n");
    output.push_str("== Summary ==\n");
    output.push_str(
        serde_json::to_string_pretty(&summary)
            .expect("summary should serialize")
            .as_str(),
    );
    output.push_str("\n\n== Tree JSON ==\n");
    output.push_str(
        serde_json::to_string_pretty(tree)
            .expect("tree should serialize")
            .as_str(),
    );
    output.push_str("\n\n== Conditional Diagnostics ==\n");
    for diagnostic in result.diagnostics.iter().filter(|diagnostic| {
        matches!(
            diagnostic.kind,
            crate::diagnostics::CrossFileDiagnosticKind::UnmatchedInject { .. }
        )
    }) {
        append!(
            output,
            "{:?}: {}\n",
            diagnostic.severity,
            diagnostic.message
        );
    }
    output.push_str("\n== Markdown ==\n");
    output.push_str(tree.to_markdown(analyzer.registry()).as_str());

    assert_snapshot!(output);
}

fn script_analysis(script: &str, components: &[&str]) -> vize_croquis::Croquis {
    let mut analyzer = vize_croquis::Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    for component in components {
        analyzer
            .croquis_mut()
            .used_components
            .insert(CompactString::new(*component));
    }
    analyzer.finish()
}
