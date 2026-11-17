use crate::support::fix_history_runner::check_pack;

#[test]
fn imported_refs_preserve_batch_diagnostics() {
    check_pack(
        "imported-ref-unwrap-2958",
        "fix_history_cases::imported_ref_unwrap_2958::imported_refs_preserve_batch_diagnostics",
        "57dd8e862ab9963567d6762629170bab3b464c05",
        Some(2958),
        &[
            "imported-ref-boolean-prop",
            "imported-ref-string-prop-error",
        ],
    );
}
