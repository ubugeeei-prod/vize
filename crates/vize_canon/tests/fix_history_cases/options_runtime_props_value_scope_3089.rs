use crate::support::fix_history_runner::check_pack;

#[test]
fn options_runtime_props_check_from_value_scope() {
    check_pack(
        "options-runtime-props-value-scope-3089",
        "fix_history_cases::options_runtime_props_value_scope_3089::options_runtime_props_check_from_value_scope",
        "3ed9fbc889078cbbc9b1a86fa3b5f8dfd8612dd6",
        Some(3089),
        &[
            "runtime-constructor-prop-binding",
            "runtime-constructor-prop-string-mismatch",
            "runtime-props-defaults-and-comments",
            "runtime-props-defaults-number-mismatch",
        ],
    );
}
