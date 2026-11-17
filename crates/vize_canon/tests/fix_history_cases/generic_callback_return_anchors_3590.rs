use crate::support::fix_history_runner::check_pack;

#[test]
fn generic_callback_prop_errors_anchor_at_the_callback_leaf() {
    check_pack(
        "generic-callback-return-anchors-3590",
        "fix_history_cases::generic_callback_return_anchors_3590::generic_callback_prop_errors_anchor_at_the_callback_leaf",
        "583cd63246042a43bd4879e9b374ce07c19c09a5",
        Some(3590),
        &[
            "guarded-and-loop-scope-callbacks-valid",
            "guarded-and-loop-scope-callback-returns",
            "contextual-callback-return-and-sibling",
            "union-branch-only-callback-return",
            "discriminated-union-callbacks-valid",
            "discriminated-union-callback-return",
            "declared-never-callback-prop",
            "callback-key-missing-from-selected-branch",
            "failed-sibling-inference-no-cascade",
        ],
    );
}
