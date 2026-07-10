use super::*;
use crate::diagnostics::{CrossFileDiagnosticKind, DiagnosticSeverity};

#[test]
fn test_shared_child_reports_and_renders_unmatched_parent_branch() {
    let mut analyzer = mixed_branch_analyzer();
    let provider = analyzer
        .registry()
        .get_id(Path::new("ParentA.vue"))
        .expect("provider should be registered");
    let child = analyzer
        .registry()
        .get_id(Path::new("Child.vue"))
        .expect("child should be registered");
    let result = analyzer.analyze();

    assert_eq!(result.provide_inject_matches.len(), 1);
    assert_eq!(result.provide_inject_matches[0].provider, provider);
    let unmatched = result
        .diagnostics
        .iter()
        .find(|diagnostic| {
            diagnostic.primary_file == child
                && matches!(
                    &diagnostic.kind,
                    CrossFileDiagnosticKind::UnmatchedInject { key } if key == "theme"
                )
        })
        .expect("the providerless parent branch should be diagnosed");
    assert_eq!(unmatched.severity, DiagnosticSeverity::Error);
    assert!(unmatched.message.contains("1 of 2 ancestor branches"));

    let tree = result
        .provide_inject_tree
        .as_ref()
        .expect("tree should be built");
    assert_eq!(
        tree.roots
            .iter()
            .map(|root| root.component_name.as_deref().unwrap())
            .collect::<Vec<_>>(),
        vec!["App", "ParentA"]
    );
    let unmatched_child = &tree.roots[0].children[0].children[0];
    assert_eq!(unmatched_child.component_name.as_deref(), Some("Child"));
    assert_eq!(unmatched_child.injects[0].provider, None);
    let matched_child = &tree.roots[1].children[0];
    assert_eq!(matched_child.component_name.as_deref(), Some("Child"));
    assert_eq!(matched_child.injects[0].provider, Some(provider));

    let tree_json = serde_json::to_value(tree).expect("tree should serialize");
    assert!(
        tree_json["roots"][0]["children"][0]["children"][0]["injects"][0]["provider"].is_null()
    );
    assert_eq!(
        tree_json["roots"][1]["children"][0]["injects"][0]["provider"],
        provider.as_u32()
    );
    let markdown = tree.to_markdown(analyzer.registry());
    assert_eq!(markdown.matches("✅").count(), 1);
    assert_eq!(markdown.matches("❌ _no provider_").count(), 1);

    let summary = result
        .provide_inject_tree_summary
        .expect("summary should be built");
    assert_eq!(summary.root_count, 2);
    assert_eq!(summary.node_count, 4);
    assert_eq!(summary.pass_through_component_count, 2);
    assert_eq!(summary.inject_count, 1);
    assert_eq!(summary.matched_inject_count, 0);
    assert_eq!(summary.unmatched_inject_count, 1);
    assert_eq!(summary.max_depth, 3);
}

#[test]
fn test_disjoint_branch_keys_are_partial_in_json_markdown_and_summary() {
    let mut analyzer = disjoint_key_analyzer();
    let parent_a = analyzer
        .registry()
        .get_id(Path::new("ParentA.vue"))
        .expect("ParentA should be registered");
    let parent_b = analyzer
        .registry()
        .get_id(Path::new("ParentB.vue"))
        .expect("ParentB should be registered");
    let result = analyzer.analyze();
    let tree = result
        .provide_inject_tree
        .as_ref()
        .expect("tree should be built");

    assert_eq!(result.provide_inject_matches.len(), 2);
    assert_eq!(tree.roots.len(), 1);
    assert_eq!(tree.roots[0].component_name.as_deref(), Some("App"));
    let branch_a = &tree.roots[0].children[0];
    let branch_b = &tree.roots[0].children[1];
    assert_eq!(branch_a.component_name.as_deref(), Some("ParentA"));
    assert_eq!(branch_b.component_name.as_deref(), Some("ParentB"));
    assert_eq!(
        branch_a.children[0].component_name.as_deref(),
        Some("Child")
    );
    assert_eq!(
        branch_b.children[0].component_name.as_deref(),
        Some("Child")
    );
    assert_eq!(
        branch_a.children[0]
            .injects
            .iter()
            .map(|inject| (inject.key.as_str(), inject.provider))
            .collect::<Vec<_>>(),
        vec![("theme", Some(parent_a)), ("locale", None)]
    );
    assert_eq!(
        branch_b.children[0]
            .injects
            .iter()
            .map(|inject| (inject.key.as_str(), inject.provider))
            .collect::<Vec<_>>(),
        vec![("theme", None), ("locale", Some(parent_b))]
    );

    let tree_json = serde_json::to_value(tree).expect("tree should serialize");
    let injects_a = &tree_json["roots"][0]["children"][0]["children"][0]["injects"];
    let injects_b = &tree_json["roots"][0]["children"][1]["children"][0]["injects"];
    assert_eq!(injects_a[0]["provider"], parent_a.as_u32());
    assert!(injects_a[1]["provider"].is_null());
    assert!(injects_b[0]["provider"].is_null());
    assert_eq!(injects_b[1]["provider"], parent_b.as_u32());
    assert_eq!(injects_a[1]["hasDefault"], true);

    let markdown = tree.to_markdown(analyzer.registry());
    assert_eq!(markdown.matches("✅").count(), 2);
    assert_eq!(markdown.matches("❌ _no provider_").count(), 2);
    assert_eq!(markdown.matches("(has default)").count(), 2);

    let mut partial_diagnostics = result
        .diagnostics
        .iter()
        .filter_map(|diagnostic| match &diagnostic.kind {
            CrossFileDiagnosticKind::UnmatchedInject { key } => {
                Some((key.as_str(), diagnostic.severity))
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    partial_diagnostics.sort_by_key(|(key, _)| *key);
    assert_eq!(
        partial_diagnostics,
        vec![
            ("locale", DiagnosticSeverity::Warning),
            ("theme", DiagnosticSeverity::Error)
        ]
    );

    let summary = result
        .provide_inject_tree_summary
        .expect("summary should be built");
    assert_eq!(summary.root_count, 1);
    assert_eq!(summary.node_count, 4);
    assert_eq!(summary.provider_component_count, 2);
    assert_eq!(summary.injector_component_count, 1);
    assert_eq!(summary.provide_count, 2);
    assert_eq!(summary.inject_count, 2);
    assert_eq!(summary.defaulted_inject_count, 1);
    assert_eq!(summary.matched_inject_count, 0);
    assert_eq!(summary.unmatched_inject_count, 2);
    assert_eq!(summary.max_child_fanout, 2);
    assert_eq!(summary.max_provider_consumer_count, 1);
}

fn mixed_branch_analyzer() -> CrossFileAnalyzer {
    let mut analyzer =
        CrossFileAnalyzer::new(CrossFileOptions::default().with_provide_inject(true));
    analyzer.add_file_with_analysis(
        Path::new("App.vue"),
        "",
        script_analysis(
            "// renders matched and unmatched parents",
            &["ParentA", "ParentB"],
        ),
    );
    analyzer.add_file_with_analysis(
        Path::new("ParentA.vue"),
        "",
        script_analysis(
            "import { provide } from 'vue'; provide('theme', 'dark')",
            &["Child"],
        ),
    );
    analyzer.add_file_with_analysis(
        Path::new("ParentB.vue"),
        "",
        script_analysis("// does not provide theme", &["Child"]),
    );
    analyzer.add_file_with_analysis(
        Path::new("Child.vue"),
        "",
        script_analysis(
            "import { inject } from 'vue'; const theme = inject('theme')",
            &[],
        ),
    );
    analyzer.rebuild_component_edges();
    analyzer
}

fn disjoint_key_analyzer() -> CrossFileAnalyzer {
    let mut analyzer =
        CrossFileAnalyzer::new(CrossFileOptions::default().with_provide_inject(true));
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
    analyzer.add_file_with_analysis(
        Path::new("ParentB.vue"),
        "",
        script_analysis(
            "import { provide } from 'vue'; provide('locale', 'ja')",
            &["Child"],
        ),
    );
    analyzer.add_file_with_analysis(
        Path::new("Child.vue"),
        "",
        script_analysis(
            "import { inject } from 'vue'; const theme = inject('theme'); const locale = inject('locale', 'en')",
            &[],
        ),
    );
    analyzer.rebuild_component_edges();
    analyzer
}
