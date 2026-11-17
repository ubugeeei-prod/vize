use crate::support::fix_history_runner::check_pack;

#[test]
fn nested_else_v_for_preserves_batch_diagnostics() {
    check_pack(
        "nested-v-if-narrowing-2959",
        "fix_history_cases::nested_v_if_narrowing_2959::nested_else_v_for_preserves_batch_diagnostics",
        "ac02c114ce529b6ef42e7ca7757279e1068548a6",
        Some(2959),
        &["nested-else-v-for", "nested-else-v-for-property-error"],
    );
}
