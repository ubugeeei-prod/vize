use super::*;

#[test]
fn test_provide_services_keep_ref_and_function_members_live() {
    use crate::rules::CrossFileReactivityIssueKind;
    use std::collections::BTreeSet;

    let mut analyzer =
        CrossFileAnalyzer::new(CrossFileOptions::default().with_reactivity_tracking(true));
    analyzer.add_file_with_analysis(
        Path::new("Provider.vue"),
        "",
        script_analysis(
            r#"import { provide, ref } from 'vue'
const location = ref('North Pole')
function updateLocation() { location.value = 'South Pole' }
const { state, open } = useToast()
const snapshot = location.value

provide('docs', { location, updateLocation })
provide('destructured-composable', { state, open })
provide('whole-composable', useToast())
provide('snapshot', { location: location.value, updateLocation })
provide('snapshot-binding', { snapshot, updateLocation })
provide('unknown-call', getToast())"#,
            &["Consumer"],
        ),
    );
    analyzer.add_file_with_analysis(
        Path::new("Consumer.vue"),
        "",
        script_analysis(
            r#"import { inject } from 'vue'
const docs = inject('docs')
const destructuredComposable = inject('destructured-composable')
const wholeComposable = inject('whole-composable')
const snapshot = inject('snapshot')
const snapshotBinding = inject('snapshot-binding')
const unknownCall = inject('unknown-call')"#,
            &[],
        ),
    );
    analyzer.rebuild_component_edges();

    let reported = analyzer
        .analyze()
        .cross_file_reactivity_issues
        .iter()
        .filter_map(|issue| match &issue.kind {
            CrossFileReactivityIssueKind::NonReactiveProvide { key } => {
                Some(key.as_str().to_owned())
            }
            _ => None,
        })
        .collect::<BTreeSet<_>>();

    assert_eq!(
        reported,
        BTreeSet::from([
            "snapshot".to_owned(),
            "snapshot-binding".to_owned(),
            "unknown-call".to_owned()
        ])
    );
}

#[test]
fn test_shared_child_reactivity_loss_reports_each_provider_context() {
    use crate::rules::CrossFileReactivityIssueKind;

    let mut analyzer =
        CrossFileAnalyzer::new(CrossFileOptions::default().with_reactivity_tracking(true));

    let parent_a = analyzer.add_file_with_analysis(
        Path::new("ParentA.vue"),
        "",
        script_analysis(
            r#"import { provide, reactive } from 'vue'
const state = reactive({ count: 1 })
provide('state', state)"#,
            &["Child"],
        ),
    );
    let parent_b = analyzer.add_file_with_analysis(
        Path::new("ParentB.vue"),
        "",
        script_analysis(
            r#"import { provide, reactive } from 'vue'
const state = reactive({ count: 2 })
provide('state', state)"#,
            &["Child"],
        ),
    );
    analyzer.add_file_with_analysis(
        Path::new("Child.vue"),
        "",
        script_analysis(
            r#"import { inject } from 'vue'
const { count } = inject('state') as { count: number }"#,
            &[],
        ),
    );
    analyzer.add_file_with_analysis(
        Path::new("App.vue"),
        "",
        script_analysis("// renders both provider branches", &["ParentA", "ParentB"]),
    );
    analyzer.rebuild_component_edges();

    let result = analyzer.analyze();
    let reactivity_losses = result
        .cross_file_reactivity_issues
        .iter()
        .filter(|issue| {
            matches!(
                &issue.kind,
                CrossFileReactivityIssueKind::InjectValueDestructured { key, .. } if key == "state"
            )
        })
        .collect::<Vec<_>>();

    assert_eq!(
        reactivity_losses.len(),
        2,
        "shared child destructuring should be tracked for each provider context"
    );
    assert!(
        reactivity_losses
            .iter()
            .any(|issue| issue.related_file == Some(parent_a))
    );
    assert!(
        reactivity_losses
            .iter()
            .any(|issue| issue.related_file == Some(parent_b))
    );
}

#[test]
fn test_reactivity_tracking_uses_later_same_component_provide() {
    use crate::rules::CrossFileReactivityIssueKind;

    let mut analyzer =
        CrossFileAnalyzer::new(CrossFileOptions::default().with_reactivity_tracking(true));
    analyzer.add_file_with_analysis(
        Path::new("Parent.vue"),
        "",
        script_analysis(
            r#"import { provide, ref } from 'vue'
const staleState = { count: 0 }
const liveState = ref({ count: 1 })
provide('state', staleState)
provide('state', liveState)"#,
            &["Child"],
        ),
    );
    analyzer.add_file_with_analysis(
        Path::new("Child.vue"),
        "",
        script_analysis(
            r#"import { inject } from 'vue'
const state = inject('state')"#,
            &[],
        ),
    );
    analyzer.rebuild_component_edges();

    let result = analyzer.analyze();
    assert!(result.cross_file_reactivity_issues.iter().all(|issue| {
        !matches!(
            &issue.kind,
            CrossFileReactivityIssueKind::NonReactiveProvide { key } if key == "state"
        )
    }));
}

#[test]
fn test_provide_inject_does_not_match_string_and_symbol_keys() {
    use crate::diagnostics::CrossFileDiagnosticKind;

    let mut analyzer =
        CrossFileAnalyzer::new(CrossFileOptions::default().with_provide_inject(true));

    let mut parent_analyzer = vize_croquis::Analyzer::with_options(AnalyzerOptions::full());
    parent_analyzer.analyze_script_setup(
        r#"import { provide, ref } from 'vue'
const ThemeKey = Symbol('theme')
const theme = ref('dark')
provide(ThemeKey, theme)"#,
    );
    parent_analyzer
        .croquis_mut()
        .note_used_component(vize_carton::CompactString::new("Child"));
    let parent_analysis = parent_analyzer.finish();

    let mut child_analyzer = vize_croquis::Analyzer::with_options(AnalyzerOptions::full());
    child_analyzer.analyze_script_setup(
        r#"import { inject } from 'vue'
const theme = inject('ThemeKey')"#,
    );
    let child_analysis = child_analyzer.finish();

    analyzer.add_file_with_analysis(Path::new("Parent.vue"), "", parent_analysis);
    analyzer.add_file_with_analysis(Path::new("Child.vue"), "", child_analysis);
    analyzer.rebuild_component_edges();

    let result = analyzer.analyze();
    assert!(
        result.provide_inject_matches.is_empty(),
        "string keys must not match symbol keys with the same display text"
    );
    assert!(result.diagnostics.iter().any(|diagnostic| {
        matches!(
            &diagnostic.kind,
            CrossFileDiagnosticKind::UnmatchedInject { key } if key == "ThemeKey"
        )
    }));
}
