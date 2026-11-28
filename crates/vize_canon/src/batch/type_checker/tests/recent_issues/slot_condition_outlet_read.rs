use super::super::{resolve_test_tsgo_binary, snapshot_project_diagnostics};
use super::real_vue::create_project_case;

/// A `v-if` / `v-else-if` that tests a declared slot and renders it through a
/// `<slot>` outlet in its branch, or on the outlet itself, calls that slot, so
/// it is not `TS2774`. A condition whose branch renders no outlet still is, as
/// in vue-tsc.
#[test]
fn slot_conditions_that_render_their_slot_are_not_uncalled_function_checks() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let root = create_project_case(
        "slot-condition-outlet-read",
        &[
            (
                "tsconfig.json",
                include_str!(
                    "../../../../../../../tests/_fixtures/differential/typechecker/slot-condition-outlet-read/tsconfig.json.txt"
                ),
            ),
            (
                "src/SlotGuard.vue",
                include_str!(
                    "../../../../../../../tests/_fixtures/differential/typechecker/slot-condition-outlet-read/SlotGuard.vue.txt"
                ),
            ),
        ],
    );
    let diagnostics = snapshot_project_diagnostics(&root).unwrap();
    std::fs::remove_dir_all(root).unwrap();
    let diagnostics: Vec<_> = diagnostics
        .iter()
        .map(|(file, code, message)| (file.as_str(), *code, message.as_str()))
        .collect();
    assert_eq!(
        diagnostics,
        vec![(
            "src/SlotGuard.vue",
            Some(2774),
            "16:14:error This condition will always return true since this function is always defined. Did you mean to call it instead?",
        )]
    );
}
