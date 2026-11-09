use crate::support::fix_history_runner::check_pack;

#[test]
fn ambient_augmentations_preserve_batch_diagnostics() {
    check_pack(
        "ambient-declarations-3095",
        "fix_history_cases::ambient_declarations_3095::ambient_augmentations_preserve_batch_diagnostics",
        "9129f14b5fa108e71cb5deed590f68f39054c1c2",
        Some(3095),
        &["global-and-module-augmentations"],
    );
}
