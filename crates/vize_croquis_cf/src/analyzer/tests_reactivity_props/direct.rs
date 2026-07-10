use super::*;
use crate::diagnostics::CrossFileDiagnosticKind;

#[test]
fn test_reactive_prop_direct_define_props_destructure_preserves_cross_file_flow() {
    let (mut analyzer, parent_id, child_id) = analyzer_with_parent_child(
        r#"import { reactive } from 'vue'
import Child from './Child.vue'
const state = reactive({ count: 0 })"#,
        r#"const { item } = defineProps<{ item: { count: number } }>()"#,
        &[("Child", &[("item", "state")])],
    );

    let result = analyzer.analyze();
    assert!(!result.cross_file_reactivity_issues.iter().any(|issue| {
        issue.file_id == child_id
            && issue.related_file == Some(parent_id)
            && matches!(
                &issue.kind,
                CrossFileReactivityIssueKind::ReactivityLostInPropChain { prop_name, .. }
                    if prop_name == "item"
            )
    }));
    assert!(!result.diagnostics.iter().any(|diagnostic| {
        diagnostic.primary_file == child_id
            && diagnostic
                .related_files
                .iter()
                .any(|(file_id, _, _)| *file_id == parent_id)
            && matches!(
                &diagnostic.kind,
                CrossFileDiagnosticKind::DestructuringBreaksReactivity { .. }
            )
    }));
}

#[test]
fn test_reactive_prop_aliased_default_define_props_destructure_preserves_cross_file_flow() {
    let (mut analyzer, parent_id, child_id) = analyzer_with_parent_child(
        r#"import { ref } from 'vue'
import Child from './Child.vue'
const selected = ref({ id: 1 })"#,
        r#"const { item: selectedItem = { id: 0 } } = defineProps<{ item?: { id: number } }>()"#,
        &[("Child", &[("item", "selected")])],
    );

    let result = analyzer.analyze();
    assert!(!result.cross_file_reactivity_issues.iter().any(|issue| {
        issue.file_id == child_id
            && issue.related_file == Some(parent_id)
            && matches!(
                &issue.kind,
                CrossFileReactivityIssueKind::ReactivityLostInPropChain { prop_name, .. }
                    if prop_name == "item"
            )
    }));
    assert!(!result.diagnostics.iter().any(|diagnostic| {
        diagnostic.primary_file == child_id
            && diagnostic
                .related_files
                .iter()
                .any(|(file_id, _, _)| *file_id == parent_id)
            && matches!(
                &diagnostic.kind,
                CrossFileDiagnosticKind::DestructuringBreaksReactivity { .. }
            )
    }));
}

#[test]
fn test_reactive_prop_indirect_props_alias_destructure_is_tracked_by_prop_key() {
    let (mut analyzer, parent_id, child_id) = analyzer_with_parent_child(
        r#"import { ref } from 'vue'
import Child from './Child.vue'
const selected = ref({ id: 1 })"#,
        r#"const props = defineProps<{ item: { id: number } }>()
const { item: selectedItem } = props"#,
        &[("Child", &[("item", "selected")])],
    );

    let result = analyzer.analyze();
    assert!(result.cross_file_reactivity_issues.iter().any(|issue| {
        issue.file_id == child_id
            && issue.related_file == Some(parent_id)
            && matches!(
                &issue.kind,
                CrossFileReactivityIssueKind::ReactivityLostInPropChain { prop_name, .. }
                    if prop_name == "item"
            )
    }));
}

#[test]
fn test_reactive_prop_member_extraction_is_cross_file_loss() {
    let (mut analyzer, parent_id, child_id) = analyzer_with_parent_child(
        r#"import { computed } from 'vue'
import Child from './Child.vue'
const total = computed(() => 1)"#,
        r#"const props = defineProps<{ total: number }>()
const localTotal = props.total"#,
        &[("Child", &[("total", "total")])],
    );

    let result = analyzer.analyze();
    assert!(result.cross_file_reactivity_issues.iter().any(|issue| {
        issue.file_id == child_id
            && issue.related_file == Some(parent_id)
            && matches!(
                &issue.kind,
                CrossFileReactivityIssueKind::ReactivityLostInPropChain { prop_name, .. }
                    if prop_name == "total"
            )
    }));
}

#[test]
fn test_reactive_prop_function_argument_is_cross_file_loss() {
    let (mut analyzer, parent_id, child_id) = analyzer_with_parent_child(
        r#"import { reactive } from 'vue'
import Child from './Child.vue'
const item = reactive({ count: 0 })"#,
        r#"const props = defineProps<{ item: { count: number } }>()
const ctx = useMyComposable(props.item)"#,
        &[("Child", &[("item", "item")])],
    );

    let result = analyzer.analyze();
    assert!(result.cross_file_reactivity_issues.iter().any(|issue| {
        issue.file_id == child_id
            && issue.related_file == Some(parent_id)
            && matches!(
                &issue.kind,
                CrossFileReactivityIssueKind::ReactivityLostInPropChain { prop_name, .. }
                    if prop_name == "item"
            )
    }));
    assert!(result.diagnostics.iter().any(|diagnostic| {
        diagnostic.primary_file == child_id
            && matches!(
                &diagnostic.kind,
                CrossFileDiagnosticKind::ValueExtractionBreaksReactivity { .. }
            )
    }));
}

#[test]
fn test_reactive_prop_plain_alias_mutation_is_cross_file_loss() {
    let (mut analyzer, parent_id, child_id) = analyzer_with_parent_child(
        r#"import { reactive } from 'vue'
import Child from './Child.vue'
const item = reactive({ count: 0 })"#,
        r#"const { item } = defineProps<{ item: { count: number } }>()
const local = item
local.count++"#,
        &[("Child", &[("item", "item")])],
    );

    let result = analyzer.analyze();
    assert!(result.cross_file_reactivity_issues.iter().any(|issue| {
        issue.file_id == child_id
            && issue.related_file == Some(parent_id)
            && matches!(
                &issue.kind,
                CrossFileReactivityIssueKind::ReactivityLostInPropChain { prop_name, .. }
                    if prop_name == "item"
            )
    }));
    assert!(result.diagnostics.iter().any(|diagnostic| {
        diagnostic.primary_file == child_id
            && matches!(
                &diagnostic.kind,
                CrossFileDiagnosticKind::ValueExtractionBreaksReactivity {
                    extracted_value,
                    ..
                } if extracted_value == "local.count"
            )
    }));
}

#[test]
fn test_reactive_prop_alias_chain_is_cross_file_loss() {
    let (mut analyzer, parent_id, child_id) = analyzer_with_parent_child(
        r#"import { reactive } from 'vue'
import Child from './Child.vue'
const item = reactive({ count: 0 })"#,
        r#"const props = defineProps<{ item: { count: number } }>()
const local = props.item
const alias = local
let assigned
assigned = alias
useMyComposable(assigned)"#,
        &[("Child", &[("item", "item")])],
    );

    let result = analyzer.analyze();
    assert!(result.cross_file_reactivity_issues.iter().any(|issue| {
        issue.file_id == child_id
            && issue.related_file == Some(parent_id)
            && matches!(
                &issue.kind,
                CrossFileReactivityIssueKind::ReactivityLostInPropChain { prop_name, .. }
                    if prop_name == "item"
            )
    }));
    assert!(result.diagnostics.iter().any(|diagnostic| {
        diagnostic.primary_file == child_id
            && matches!(
                &diagnostic.kind,
                CrossFileDiagnosticKind::ValueExtractionBreaksReactivity {
                    extracted_value,
                    ..
                } if extracted_value == "assigned"
            )
    }));
}

#[test]
fn test_reactive_prop_getter_context_extraction_is_cross_file_loss() {
    let (mut analyzer, parent_id, child_id) = analyzer_with_parent_child(
        r#"import { computed } from 'vue'
import Child from './Child.vue'
const item = computed(() => ({ count: 0 }))"#,
        r#"const props = defineProps<{ item: { count: number } }>()
const ctx = useMyComposable(() => props.item)
const localItem = ctx.item()"#,
        &[("Child", &[("item", "item")])],
    );

    let result = analyzer.analyze();
    assert!(result.cross_file_reactivity_issues.iter().any(|issue| {
        issue.file_id == child_id
            && issue.related_file == Some(parent_id)
            && matches!(
                &issue.kind,
                CrossFileReactivityIssueKind::ReactivityLostInPropChain { prop_name, .. }
                    if prop_name == "item"
            )
    }));
    assert!(result.diagnostics.iter().any(|diagnostic| {
        diagnostic.primary_file == child_id
            && matches!(
                &diagnostic.kind,
                CrossFileDiagnosticKind::ValueExtractionBreaksReactivity { .. }
            )
    }));
}

#[test]
fn test_nested_reactive_prop_member_extraction_tracks_root_prop() {
    let (mut analyzer, parent_id, child_id) = analyzer_with_parent_child(
        r#"import { reactive } from 'vue'
import Child from './Child.vue'
const user = reactive({ profile: { name: 'A' } })"#,
        r#"const props = defineProps<{ user: { profile: { name: string } } }>()
const localName = props.user.profile.name"#,
        &[("Child", &[("user", "user")])],
    );

    let result = analyzer.analyze();
    assert!(result.cross_file_reactivity_issues.iter().any(|issue| {
        issue.file_id == child_id
            && issue.related_file == Some(parent_id)
            && matches!(
                &issue.kind,
                CrossFileReactivityIssueKind::ReactivityLostInPropChain { prop_name, .. }
                    if prop_name == "user"
            )
    }));
}
