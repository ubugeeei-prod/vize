//! Event-prop inference is shared only while its declaration remains visible
//! across sibling listeners on one guarded component (#12005).

use super::super::{resolve_test_tsgo_binary, snapshot_project_diagnostics};

#[test]
fn guarded_call_listener_does_not_hide_later_arrow_context() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project = super::real_vue::create_project_case(
        "component-listener-guard-share-12005",
        &[
            (
                "Child.vue",
                include_str!(
                    "../../../../../../../tests/_fixtures/differential/typechecker/component-listener-guard-share-12005/Child.vue.txt"
                ),
            ),
            (
                "Probe.vue",
                include_str!(
                    "../../../../../../../tests/_fixtures/differential/typechecker/component-listener-guard-share-12005/Probe.vue.txt"
                ),
            ),
            (
                "package.json",
                include_str!(
                    "../../../../../../../tests/_fixtures/differential/typechecker/component-listener-guard-share-12005/package.json.txt"
                ),
            ),
            (
                "tsconfig.json",
                include_str!(
                    "../../../../../../../tests/_fixtures/differential/typechecker/component-listener-guard-share-12005/tsconfig.json.txt"
                ),
            ),
        ],
    );
    let actual = snapshot_project_diagnostics(&project);
    let _ = std::fs::remove_dir_all(project);
    assert_eq!(
        actual,
        Some(Vec::new()),
        "the later listener must keep its declared string context",
    );
}

#[test]
fn guarded_reference_listener_does_not_hide_later_arrow_context() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project = super::real_vue::create_project_case(
        "component-listener-reference-guard-share-12017",
        &[
            (
                "Child.vue",
                include_str!(
                    "../../../../../../../tests/_fixtures/differential/typechecker/component-listener-reference-guard-share-12017/Child.vue.txt"
                ),
            ),
            (
                "Probe.vue",
                include_str!(
                    "../../../../../../../tests/_fixtures/differential/typechecker/component-listener-reference-guard-share-12017/Probe.vue.txt"
                ),
            ),
            (
                "package.json",
                include_str!(
                    "../../../../../../../tests/_fixtures/differential/typechecker/component-listener-reference-guard-share-12017/package.json.txt"
                ),
            ),
            (
                "tsconfig.json",
                include_str!(
                    "../../../../../../../tests/_fixtures/differential/typechecker/component-listener-reference-guard-share-12017/tsconfig.json.txt"
                ),
            ),
        ],
    );
    let actual = snapshot_project_diagnostics(&project);
    let _ = std::fs::remove_dir_all(project);
    assert_eq!(
        actual,
        Some(Vec::new()),
        "a bare listener reference must not hide a sibling listener's declared context",
    );
}
