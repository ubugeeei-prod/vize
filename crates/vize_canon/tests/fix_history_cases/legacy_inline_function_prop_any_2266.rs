use crate::support::fix_history_runner::check_pack;

#[test]
fn legacy_library_inline_function_props_skip_implicit_any() {
    check_pack(
        "legacy-inline-function-prop-any-2266",
        "fix_history_cases::legacy_inline_function_prop_any_2266::legacy_library_inline_function_props_skip_implicit_any",
        "f4d945bb5263b198749bc6cfbe3f24272dc57c60",
        Some(2266),
        &[
            "inline-rule-callback-no-implicit-any",
            "sibling-prop-expression-still-checked",
        ],
    );
}
