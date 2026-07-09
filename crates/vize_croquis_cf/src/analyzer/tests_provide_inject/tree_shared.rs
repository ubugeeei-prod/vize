use super::*;

#[test]
fn test_shared_child_resolves_each_parent_provider_context() {
    use crate::diagnostics::CrossFileDiagnosticKind;

    let mut analyzer = shared_child_analyzer(&["App", "ParentA", "ParentB", "Child"]);
    let parent_a = analyzer
        .registry()
        .get_id(Path::new("ParentA.vue"))
        .expect("ParentA should be registered");
    let parent_b = analyzer
        .registry()
        .get_id(Path::new("ParentB.vue"))
        .expect("ParentB should be registered");
    let child = analyzer
        .registry()
        .get_id(Path::new("Child.vue"))
        .expect("Child should be registered");

    let result = analyzer.analyze();
    let mut theme_matches = result
        .provide_inject_matches
        .iter()
        .filter(|provider_match| provider_match.key == "theme")
        .collect::<Vec<_>>();
    theme_matches.sort_by_key(|provider_match| provider_match.provider.as_u32());

    assert_eq!(
        theme_matches.len(),
        2,
        "a reused child should be matched in each parent render context"
    );
    assert!(
        theme_matches
            .iter()
            .any(|provider_match| provider_match.provider == parent_a)
    );
    assert!(
        theme_matches
            .iter()
            .any(|provider_match| provider_match.provider == parent_b)
    );
    assert!(
        theme_matches
            .iter()
            .all(|provider_match| provider_match.consumer == child)
    );
    assert!(result.diagnostics.iter().all(|diagnostic| {
        !matches!(
            diagnostic.kind,
            CrossFileDiagnosticKind::UnmatchedInject { .. }
                | CrossFileDiagnosticKind::UnusedProvide { .. }
        )
    }));

    let tree = result
        .provide_inject_tree
        .as_ref()
        .expect("tree should be built");
    assert_eq!(tree.roots.len(), 2);
    assert!(tree.roots.iter().all(|root| {
        root.children.len() == 1
            && root.children[0].file_id == child
            && root.children[0].injects.len() == 1
            && root.children[0].injects[0].provider == Some(root.file_id)
    }));

    let summary = result
        .provide_inject_tree_summary
        .expect("tree summary should be built");
    assert_eq!(summary.root_count, 2);
    assert_eq!(summary.node_count, 3, "shared components count once");
    assert_eq!(summary.leaf_component_count, 1);
    assert_eq!(summary.provider_component_count, 2);
    assert_eq!(summary.injector_component_count, 1);
    assert_eq!(summary.provide_count, 2);
    assert_eq!(summary.inject_count, 1, "shared inject calls count once");
    assert_eq!(summary.matched_inject_count, 1);
    assert_eq!(summary.unmatched_inject_count, 0);
    assert_eq!(summary.max_depth, 2);
    assert_eq!(summary.max_child_fanout, 1);
    assert_eq!(summary.max_provider_consumer_count, 1);
}

#[test]
fn test_shared_child_tree_is_deterministic_when_registration_order_is_reversed() {
    let mut forward = shared_child_analyzer(&["App", "ParentA", "ParentB", "Child"]);
    let forward_result = forward.analyze();
    let forward_tree = forward_result
        .provide_inject_tree
        .as_ref()
        .expect("forward tree should be built");
    assert_branch_local_providers(forward_tree);

    let mut reversed = shared_child_analyzer(&["Child", "ParentB", "ParentA", "App"]);
    let reversed_result = reversed.analyze();
    let reversed_tree = reversed_result
        .provide_inject_tree
        .as_ref()
        .expect("reversed tree should be built");
    assert_branch_local_providers(reversed_tree);

    assert_eq!(
        forward_tree.to_markdown(forward.registry()),
        reversed_tree.to_markdown(reversed.registry()),
        "rendering should be ordered by stable paths, not registration IDs"
    );
    assert_eq!(
        branch_provider_names(forward_tree, forward.registry()),
        branch_provider_names(reversed_tree, reversed.registry())
    );
    assert_eq!(
        forward_result.provide_inject_tree_summary,
        reversed_result.provide_inject_tree_summary
    );
}

fn shared_child_analyzer(registration_order: &[&str]) -> CrossFileAnalyzer {
    let mut analyzer =
        CrossFileAnalyzer::new(CrossFileOptions::default().with_provide_inject(true));

    for component in registration_order {
        let (path, analysis) = match *component {
            "App" => (
                "App.vue",
                script_analysis("// renders both parents", &["ParentA", "ParentB"]),
            ),
            "ParentA" => (
                "ParentA.vue",
                script_analysis(
                    r#"import { provide, ref } from 'vue'
const theme = ref('dark')
provide('theme', theme)"#,
                    &["Child"],
                ),
            ),
            "ParentB" => (
                "ParentB.vue",
                script_analysis(
                    r#"import { provide, ref } from 'vue'
const theme = ref('light')
provide('theme', theme)"#,
                    &["Child"],
                ),
            ),
            "Child" => (
                "Child.vue",
                script_analysis(
                    r#"import { inject } from 'vue'
const theme = inject('theme')"#,
                    &[],
                ),
            ),
            unexpected => panic!("unexpected shared-child component: {unexpected}"),
        };
        analyzer.add_file_with_analysis(Path::new(path), "", analysis);
    }

    analyzer.rebuild_component_edges();
    analyzer
}

fn assert_branch_local_providers(tree: &crate::rules::ProvideInjectTree) {
    assert_eq!(tree.roots.len(), 2);
    for root in &tree.roots {
        assert_eq!(root.children.len(), 1);
        let child = &root.children[0];
        assert_eq!(child.injects.len(), 1);
        assert_eq!(child.injects[0].provider, Some(root.file_id));
    }
}

fn branch_provider_names(
    tree: &crate::rules::ProvideInjectTree,
    registry: &crate::registry::ModuleRegistry,
) -> Vec<(vize_carton::CompactString, vize_carton::CompactString)> {
    tree.roots
        .iter()
        .map(|root| {
            let root_name = root
                .component_name
                .clone()
                .expect("provider root should have a component name");
            let provider_id = root.children[0].injects[0]
                .provider
                .expect("shared child should resolve a provider");
            let provider_name = registry
                .get(provider_id)
                .and_then(|entry| entry.component_name.clone())
                .expect("resolved provider should have a component name");
            (root_name, provider_name)
        })
        .collect()
}
