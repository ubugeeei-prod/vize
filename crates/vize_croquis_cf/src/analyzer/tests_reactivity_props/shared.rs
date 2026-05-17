use super::*;

#[test]
fn test_shared_child_reactive_prop_loss_reports_each_parent_context() {
    let mut analyzer =
        CrossFileAnalyzer::new(CrossFileOptions::default().with_reactivity_tracking(true));
    let parent_a = analyzer.add_file_with_analysis(
        Path::new("ParentA.vue"),
        "",
        script_analysis(
            r#"import { reactive } from 'vue'
import Child from './Child.vue'
const state = reactive({ count: 1 })"#,
            &[("Child", &[("item", "state")])],
        ),
    );
    let parent_b = analyzer.add_file_with_analysis(
        Path::new("ParentB.vue"),
        "",
        script_analysis(
            r#"import { reactive } from 'vue'
import Child from './Child.vue'
const state = reactive({ count: 2 })"#,
            &[("Child", &[("item", "state")])],
        ),
    );
    let child = analyzer.add_file_with_analysis(
        Path::new("Child.vue"),
        "",
        script_analysis(
            r#"const props = defineProps<{ item: { count: number } }>()
const { item } = props"#,
            &[],
        ),
    );
    analyzer.rebuild_import_edges();
    analyzer.rebuild_component_edges();

    let result = analyzer.analyze();
    let losses = result
        .cross_file_reactivity_issues
        .iter()
        .filter(|issue| {
            issue.file_id == child
                && matches!(
                    &issue.kind,
                    CrossFileReactivityIssueKind::ReactivityLostInPropChain { prop_name, .. }
                        if prop_name == "item"
                )
        })
        .collect::<Vec<_>>();

    assert_eq!(losses.len(), 2);
    assert!(
        losses
            .iter()
            .any(|issue| issue.related_file == Some(parent_a))
    );
    assert!(
        losses
            .iter()
            .any(|issue| issue.related_file == Some(parent_b))
    );
}

#[test]
fn test_non_reactive_parent_prop_does_not_create_prop_chain_loss() {
    let (mut analyzer, _parent_id, _child_id) = analyzer_with_parent_child(
        r#"import Child from './Child.vue'
const label = 'static'"#,
        r#"const props = defineProps<{ label: string }>()
const localLabel = props.label"#,
        &[("Child", &[("label", "label")])],
    );

    let result = analyzer.analyze();
    assert!(result.cross_file_reactivity_issues.iter().all(|issue| {
        !matches!(
            &issue.kind,
            CrossFileReactivityIssueKind::ReactivityLostInPropChain { prop_name, .. }
                if prop_name == "label"
        )
    }));
}

#[test]
fn test_to_ref_props_consumption_preserves_cross_file_reactivity() {
    let (mut analyzer, _parent_id, _child_id) = analyzer_with_parent_child(
        r#"import { reactive } from 'vue'
import Child from './Child.vue'
const state = reactive({ count: 0 })"#,
        r#"import { toRef } from 'vue'
const props = defineProps<{ item: { count: number } }>()
const item = toRef(props, 'item')"#,
        &[("Child", &[("item", "state")])],
    );

    let result = analyzer.analyze();
    assert!(result.cross_file_reactivity_issues.iter().all(|issue| {
        !matches!(
            &issue.kind,
            CrossFileReactivityIssueKind::ReactivityLostInPropChain { prop_name, .. }
                if prop_name == "item"
        )
    }));
}

#[test]
fn test_component_usage_matching_does_not_leak_props_to_sibling_child() {
    let mut analyzer =
        CrossFileAnalyzer::new(CrossFileOptions::default().with_reactivity_tracking(true));
    analyzer.add_file_with_analysis(
        Path::new("Parent.vue"),
        "",
        script_analysis(
            r#"import { reactive } from 'vue'
import SafeChild from './SafeChild.vue'
import RiskyChild from './RiskyChild.vue'
const state = reactive({ count: 0 })"#,
            &[("SafeChild", &[("item", "state")])],
        ),
    );
    analyzer.add_file_with_analysis(
        Path::new("SafeChild.vue"),
        "",
        script_analysis(
            r#"import { toRef } from 'vue'
const props = defineProps<{ item: { count: number } }>()
const item = toRef(props, 'item')"#,
            &[],
        ),
    );
    let risky_child = analyzer.add_file_with_analysis(
        Path::new("RiskyChild.vue"),
        "",
        script_analysis(
            r#"const props = defineProps<{ item: { count: number } }>()
const { item } = props"#,
            &[],
        ),
    );
    analyzer.rebuild_import_edges();
    analyzer.rebuild_component_edges();

    let result = analyzer.analyze();
    assert!(result.cross_file_reactivity_issues.iter().all(|issue| {
        issue.file_id != risky_child
            || !matches!(
                &issue.kind,
                CrossFileReactivityIssueKind::ReactivityLostInPropChain { prop_name, .. }
                    if prop_name == "item"
            )
    }));
}
