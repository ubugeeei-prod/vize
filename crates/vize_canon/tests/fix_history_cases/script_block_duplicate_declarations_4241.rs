use crate::support::fix_history_runner::check_pack;

#[test]
fn script_block_collisions_preserve_batch_diagnostics() {
    check_pack(
        "script-block-duplicate-declarations-4241",
        "fix_history_cases::script_block_duplicate_declarations_4241::script_block_collisions_preserve_batch_diagnostics",
        "da1de3654e840605f65b18e6da536340cc6a28b4",
        Some(4241),
        &[
            "collisions-and-shadowing",
            "setup-type-over-classic-value",
            "repaired-classic-duplicate",
        ],
    );
}
