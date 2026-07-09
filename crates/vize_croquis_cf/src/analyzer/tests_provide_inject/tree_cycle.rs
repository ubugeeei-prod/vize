use super::*;

#[test]
fn test_cyclic_provider_graph_selects_stable_roots_without_recursing() {
    let mut forward = cycle_analyzer(&["A", "B"]);
    let forward_result = forward.analyze();
    let mut reversed = cycle_analyzer(&["B", "A"]);
    let reversed_result = reversed.analyze();

    assert_eq!(forward_result.provide_inject_matches.len(), 2);
    assert_eq!(reversed_result.provide_inject_matches.len(), 2);
    let forward_tree = forward_result
        .provide_inject_tree
        .as_ref()
        .expect("forward cycle tree should be built");
    let reversed_tree = reversed_result
        .provide_inject_tree
        .as_ref()
        .expect("reversed cycle tree should be built");
    assert_cycle_tree(forward_tree);
    assert_cycle_tree(reversed_tree);
    assert_eq!(
        forward_tree.to_markdown(forward.registry()),
        reversed_tree.to_markdown(reversed.registry())
    );

    let tree_json = serde_json::to_value(forward_tree).expect("cycle tree should serialize");
    assert_eq!(tree_json["roots"].as_array().unwrap().len(), 2);
    assert!(
        tree_json["roots"][0]["children"][0]["children"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert!(
        tree_json["roots"][1]["children"][0]["children"]
            .as_array()
            .unwrap()
            .is_empty()
    );

    let summary = forward_result
        .provide_inject_tree_summary
        .expect("cycle summary should be built");
    assert_eq!(summary.root_count, 2);
    assert_eq!(summary.node_count, 2);
    assert_eq!(summary.provider_component_count, 2);
    assert_eq!(summary.injector_component_count, 2);
    assert_eq!(summary.provide_count, 2);
    assert_eq!(summary.inject_count, 2);
    assert_eq!(summary.matched_inject_count, 2);
    assert_eq!(summary.unmatched_inject_count, 0);
    assert_eq!(summary.max_depth, 2);
    assert_eq!(summary.max_child_fanout, 1);
    assert_eq!(summary.max_provider_consumer_count, 1);
    assert_eq!(
        forward_result.provide_inject_tree_summary,
        reversed_result.provide_inject_tree_summary
    );
}

fn cycle_analyzer(registration_order: &[&str]) -> CrossFileAnalyzer {
    let mut analyzer =
        CrossFileAnalyzer::new(CrossFileOptions::default().with_provide_inject(true));
    for component in registration_order {
        let (path, analysis) = match *component {
            "A" => (
                "A.vue",
                script_analysis(
                    "import { provide, inject } from 'vue'; provide('fromA', 1); const fromB = inject('fromB')",
                    &["B"],
                ),
            ),
            "B" => (
                "B.vue",
                script_analysis(
                    "import { provide, inject } from 'vue'; provide('fromB', 2); const fromA = inject('fromA')",
                    &["A"],
                ),
            ),
            unexpected => panic!("unexpected cycle component: {unexpected}"),
        };
        analyzer.add_file_with_analysis(Path::new(path), "", analysis);
    }
    analyzer.rebuild_component_edges();
    analyzer
}

fn assert_cycle_tree(tree: &crate::rules::ProvideInjectTree) {
    assert_eq!(
        tree.roots
            .iter()
            .map(|root| root.component_name.as_deref().unwrap())
            .collect::<Vec<_>>(),
        vec!["A", "B"]
    );
    for root in &tree.roots {
        assert_eq!(root.provides.len(), 1);
        assert!(root.injects.is_empty());
        assert_eq!(root.children.len(), 1);
        let child = &root.children[0];
        assert_eq!(child.provides.len(), 1);
        assert_eq!(child.injects.len(), 1);
        assert_eq!(child.injects[0].provider, Some(root.file_id));
        assert!(child.children.is_empty());
    }
}
