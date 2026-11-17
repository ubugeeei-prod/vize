use crate::support::fix_history_runner::check_pack;

#[test]
fn plain_script_value_and_type_exports_preserve_batch_diagnostics() {
    check_pack(
        "plain-script-enum-class-exports-3382",
        "fix_history_cases::plain_script_enum_class_exports_3382::plain_script_value_and_type_exports_preserve_batch_diagnostics",
        "813a7ba187995f46cd4c3c0b8ec03104bcd9d9e3",
        Some(3382),
        &["value-and-type-exports", "enum-type-side-error"],
    );
}
