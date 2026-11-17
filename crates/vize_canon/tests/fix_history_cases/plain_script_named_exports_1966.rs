use crate::support::fix_history_runner::check_pack;

#[test]
fn plain_script_value_exports_preserve_batch_diagnostics() {
    check_pack(
        "plain-script-named-exports-1966",
        "fix_history_cases::plain_script_named_exports_1966::plain_script_value_exports_preserve_batch_diagnostics",
        "a4da0228d54f89923e2982044eeab2c4809529be",
        Some(1966),
        &["named-value-export", "named-value-export-type-error"],
    );
}
