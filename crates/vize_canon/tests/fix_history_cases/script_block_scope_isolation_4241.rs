use crate::support::fix_history_runner::check_pack;

#[test]
fn cross_block_type_sharing_preserves_batch_diagnostics() {
    check_pack(
        "script-block-scope-isolation-4241",
        "fix_history_cases::script_block_scope_isolation_4241::cross_block_type_sharing_preserves_batch_diagnostics",
        "da1de3654e840605f65b18e6da536340cc6a28b4",
        Some(4241),
        &["cross-block-type-errors", "repaired-consumer"],
    );
}
