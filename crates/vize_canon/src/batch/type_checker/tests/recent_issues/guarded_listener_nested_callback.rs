//! A render-time copy of a guarded component listener must not own nested
//! callback parameter diagnostics; the typed event scope is the owner (#11977).

use super::super::{resolve_test_tsgo_binary, snapshot_project_diagnostics};

#[test]
fn guarded_component_listener_keeps_nested_callback_context() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project = super::real_vue::create_project_case(
        "guarded-listener-nested-callback-11977",
        &[
            (
                "src/Child.vue",
                include_str!(
                    "../../../../../../../tests/_fixtures/differential/typechecker/guarded-listener-nested-callback-11977/Child.vue.txt"
                ),
            ),
            (
                "src/Parent.vue",
                include_str!(
                    "../../../../../../../tests/_fixtures/differential/typechecker/guarded-listener-nested-callback-11977/Parent.vue.txt"
                ),
            ),
        ],
    );
    let actual = snapshot_project_diagnostics(&project);
    let _ = std::fs::remove_dir_all(project);
    assert_eq!(actual, Some(Vec::new()));
}
