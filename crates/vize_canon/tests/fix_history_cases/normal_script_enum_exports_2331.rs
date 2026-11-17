use crate::support::fix_history_runner::check_pack;

#[test]
fn normal_script_ts_enum_is_a_named_value_export() {
    check_pack(
        "normal-script-enum-exports-2331",
        "fix_history_cases::normal_script_enum_exports_2331::normal_script_ts_enum_is_a_named_value_export",
        "5eb53f331a55a9fdb02bb9e8ab83c141b0918146",
        Some(2331),
        &[
            "enum-export-from-normal-script",
            "enum-export-rejects-string-literal",
        ],
    );
}
