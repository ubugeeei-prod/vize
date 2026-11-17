use crate::support::fix_history_runner::check_pack;

#[test]
fn legacy_vue2_virtual_ts_uses_structural_helpers_without_vue3_exports() {
    check_pack(
        "legacy-vue2-structural-helpers-1923",
        "fix_history_cases::legacy_vue2_structural_helpers_1923::legacy_vue2_virtual_ts_uses_structural_helpers_without_vue3_exports",
        "949fae1251fa05c96f1536e0e70d9a6d6e2f7984",
        Some(1923),
        &[
            "setup-and-options-without-vue3-helper-exports",
            "setup-ref-unwrap-wrong-member",
        ],
    );
}
