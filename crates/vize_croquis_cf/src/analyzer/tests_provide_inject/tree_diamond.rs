use super::*;

#[test]
fn test_same_provider_diamond_preserves_each_path_deterministically() {
    let mut forward = diamond_analyzer(&["Provider", "A", "B", "Child"]);
    let forward_result = forward.analyze();
    let mut reversed = diamond_analyzer(&["Child", "B", "A", "Provider"]);
    let reversed_result = reversed.analyze();

    let expected = vec![vec!["Provider", "A", "Child"]];
    assert_eq!(
        component_match_paths(
            &forward_result.provide_inject_matches,
            forward.registry(),
            "theme"
        ),
        expected
    );
    assert_eq!(
        component_match_paths(
            &reversed_result.provide_inject_matches,
            reversed.registry(),
            "theme"
        ),
        expected
    );

    let forward_tree = forward_result
        .provide_inject_tree
        .as_ref()
        .expect("forward tree should be built");
    let reversed_tree = reversed_result
        .provide_inject_tree
        .as_ref()
        .expect("reversed tree should be built");
    assert_diamond_tree(forward_tree, forward.registry());
    assert_diamond_tree(reversed_tree, reversed.registry());
    assert_eq!(
        forward_tree.to_markdown(forward.registry()),
        reversed_tree.to_markdown(reversed.registry())
    );

    let summary = forward_result
        .provide_inject_tree_summary
        .expect("summary should be built");
    assert_eq!(summary.root_count, 1);
    assert_eq!(summary.node_count, 4);
    assert_eq!(summary.leaf_component_count, 1);
    assert_eq!(summary.pass_through_component_count, 2);
    assert_eq!(summary.provider_component_count, 1);
    assert_eq!(summary.injector_component_count, 1);
    assert_eq!(summary.provide_count, 1);
    assert_eq!(summary.inject_count, 1);
    assert_eq!(summary.matched_inject_count, 1);
    assert_eq!(summary.unmatched_inject_count, 0);
    assert_eq!(summary.max_depth, 3);
    assert_eq!(summary.max_child_fanout, 2);
    assert_eq!(summary.max_provider_consumer_count, 2);
    assert_eq!(
        forward_result
            .complexity_report
            .input
            .provide_inject_reference_count,
        2,
        "source-level provide and inject calls should count once"
    );
    assert_eq!(
        forward_result
            .complexity_report
            .input
            .provide_inject_fanout_count,
        2,
        "branch occurrence fanout comes from the internal tree summary"
    );
    let provider = forward
        .registry()
        .get_id(Path::new("Provider.vue"))
        .expect("Provider should be registered");
    let provider_hotspot = forward_result
        .complexity_hotspots
        .iter()
        .find(|hotspot| hotspot.file_id == provider)
        .expect("Provider should have a complexity hotspot");
    assert_eq!(provider_hotspot.input.provide_inject_reference_count, 1);
    assert_eq!(provider_hotspot.input.provide_inject_fanout_count, 1);
    assert_eq!(
        forward_result.provide_inject_tree_summary,
        reversed_result.provide_inject_tree_summary
    );
}

fn diamond_analyzer(registration_order: &[&str]) -> CrossFileAnalyzer {
    let mut analyzer =
        CrossFileAnalyzer::new(CrossFileOptions::default().with_provide_inject(true));
    for component in registration_order {
        let (path, analysis) = match *component {
            "Provider" => (
                "Provider.vue",
                script_analysis(
                    "import { provide } from 'vue'; provide('theme', 'dark')",
                    &["A", "B"],
                ),
            ),
            "A" => ("A.vue", script_analysis("// pass through A", &["Child"])),
            "B" => ("B.vue", script_analysis("// pass through B", &["Child"])),
            "Child" => (
                "Child.vue",
                script_analysis(
                    "import { inject } from 'vue'; const theme = inject('theme')",
                    &[],
                ),
            ),
            unexpected => panic!("unexpected diamond component: {unexpected}"),
        };
        analyzer.add_file_with_analysis(Path::new(path), "", analysis);
    }
    analyzer.rebuild_component_edges();
    analyzer
}

fn component_match_paths<'a>(
    matches: &'a [crate::rules::ProvideInjectMatch],
    registry: &'a crate::registry::ModuleRegistry,
    key: &str,
) -> Vec<Vec<&'a str>> {
    matches
        .iter()
        .filter(|provider_match| provider_match.key == key)
        .map(|provider_match| {
            provider_match
                .path
                .iter()
                .map(|file_id| {
                    registry
                        .get(*file_id)
                        .and_then(|entry| entry.component_name.as_deref())
                        .expect("path component should be registered")
                })
                .collect()
        })
        .collect()
}

fn assert_diamond_tree(
    tree: &crate::rules::ProvideInjectTree,
    registry: &crate::registry::ModuleRegistry,
) {
    assert_eq!(tree.roots.len(), 1);
    let root = &tree.roots[0];
    assert_eq!(root.component_name.as_deref(), Some("Provider"));
    assert_eq!(root.provides[0].consumer_count, 2);
    assert_eq!(root.children.len(), 2);
    assert_eq!(
        root.children
            .iter()
            .map(|node| node.component_name.as_deref().unwrap())
            .collect::<Vec<_>>(),
        vec!["A", "B"]
    );
    for branch in &root.children {
        assert_eq!(branch.children.len(), 1);
        let child = &branch.children[0];
        assert_eq!(child.component_name.as_deref(), Some("Child"));
        assert_eq!(child.injects[0].provider, Some(root.file_id));
        assert_eq!(
            registry
                .get(child.injects[0].provider.unwrap())
                .and_then(|entry| entry.component_name.as_deref()),
            Some("Provider")
        );
    }
}
