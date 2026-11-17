use crate::support::fix_history_runner::check_pack;

#[test]
fn empty_interpolations_preserve_batch_diagnostics() {
    check_pack(
        "empty-template-interpolation-3090",
        "fix_history_cases::empty_template_interpolation_3090::empty_interpolations_preserve_batch_diagnostics",
        "7c18c96fe98af6b74f57753b7801e647f751d50d",
        Some(3090),
        &["empty-interpolation", "adjacent-interpolation-error"],
    );
}
