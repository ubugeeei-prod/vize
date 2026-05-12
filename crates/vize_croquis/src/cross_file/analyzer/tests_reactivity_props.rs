use super::{CrossFileAnalyzer, CrossFileOptions};
use crate::analysis::{ComponentUsage, PassedProp};
use crate::cross_file::analyzers::CrossFileReactivityIssueKind;
use crate::cross_file::diagnostics::{CrossFileDiagnosticKind, DiagnosticSeverity};
use crate::AnalyzerOptions;
use std::path::Path;
use vize_carton::{CompactString, SmallVec};

fn script_analysis(script: &str, usages: &[(&str, &[(&str, &str)])]) -> crate::Croquis {
    let mut analyzer = crate::Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);

    for (component, props) in usages {
        analyzer
            .croquis_mut()
            .used_components
            .insert(CompactString::new(*component));
        analyzer
            .croquis_mut()
            .component_usages
            .push(component_usage(component, props));
    }

    analyzer.finish()
}

fn component_usage(component: &str, props: &[(&str, &str)]) -> ComponentUsage {
    ComponentUsage {
        name: CompactString::new(component),
        start: 0,
        end: component.len() as u32,
        props: props
            .iter()
            .enumerate()
            .map(|(index, (name, value))| PassedProp {
                name: CompactString::new(*name),
                value: Some(CompactString::new(*value)),
                start: index as u32,
                end: index as u32 + name.len() as u32,
                is_dynamic: true,
            })
            .collect(),
        events: SmallVec::new(),
        slots: SmallVec::new(),
        has_spread_attrs: false,
        scope_id: crate::scope::ScopeId::ROOT,
        vif_guard: None,
    }
}

fn analyzer_with_parent_child(
    parent_script: &str,
    child_script: &str,
    usages: &[(&str, &[(&str, &str)])],
) -> (
    CrossFileAnalyzer,
    crate::cross_file::FileId,
    crate::cross_file::FileId,
) {
    let mut analyzer =
        CrossFileAnalyzer::new(CrossFileOptions::default().with_reactivity_tracking(true));
    let parent_id = analyzer.add_file_with_analysis(
        Path::new("Parent.vue"),
        "",
        script_analysis(parent_script, usages),
    );
    let child_id = analyzer.add_file_with_analysis(
        Path::new("Child.vue"),
        "",
        script_analysis(child_script, &[]),
    );
    analyzer.rebuild_import_edges();
    analyzer.rebuild_component_edges();

    (analyzer, parent_id, child_id)
}

#[test]
fn test_reactive_prop_direct_define_props_destructure_is_cross_file_error() {
    let (mut analyzer, parent_id, child_id) = analyzer_with_parent_child(
        r#"import { reactive } from 'vue'
import Child from './Child.vue'
const state = reactive({ count: 0 })"#,
        r#"const { item } = defineProps<{ item: { count: number } }>()"#,
        &[("Child", &[("item", "state")])],
    );

    let result = analyzer.analyze();
    let issue = result
        .cross_file_reactivity_issues
        .iter()
        .find(|issue| {
            issue.file_id == child_id
                && issue.related_file == Some(parent_id)
                && matches!(
                    &issue.kind,
                    CrossFileReactivityIssueKind::ReactivityLostInPropChain { prop_name, .. }
                        if prop_name == "item"
                )
        })
        .expect("reactive prop destructure should be reported");

    assert_eq!(issue.severity, DiagnosticSeverity::Error);
    assert!(result.diagnostics.iter().any(|diagnostic| {
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
    assert!(losses
        .iter()
        .any(|issue| issue.related_file == Some(parent_a)));
    assert!(losses
        .iter()
        .any(|issue| issue.related_file == Some(parent_b)));
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
