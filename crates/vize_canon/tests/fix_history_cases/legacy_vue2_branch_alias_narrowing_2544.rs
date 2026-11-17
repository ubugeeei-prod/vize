use crate::support::fix_history_runner::check_pack;

#[test]
fn legacy_vue2_v_for_branch_guards_keep_alias_narrowing() {
    check_pack(
        "legacy-vue2-branch-alias-narrowing-2544",
        "fix_history_cases::legacy_vue2_branch_alias_narrowing_2544::legacy_vue2_v_for_branch_guards_keep_alias_narrowing",
        "845356cb92313956ebb5f134f0e3f628a9d20b66",
        Some(2544),
        &[
            "nested-v-for-else-branch-alias-narrowing",
            "summary-branch-rejects-history-member",
            "pass-branch-rejects-fail-member",
        ],
    );
}
