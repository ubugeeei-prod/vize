use crate::support::fix_history_runner::check_pack;

#[test]
fn adjacent_ambient_declarations_preserve_lf_batch_diagnostics() {
    check_pack(
        "setup-ambient-adjacent-6223",
        "fix_history_cases::setup_ambient_adjacent_6223::adjacent_ambient_declarations_preserve_lf_batch_diagnostics",
        "be16d7c61b992af1e9af7e49a6975ce7a16e2512",
        Some(6223),
        &[
            "forward",
            "chain",
            "overloads",
            "class",
            "multiline",
            "assignment",
            "missing-type",
            "const-write",
            "property",
            "implicit-any",
            "union",
            "template",
        ],
    );
}
