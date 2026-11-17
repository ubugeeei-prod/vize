use crate::support::fix_history_runner::check_pack;

#[test]
fn required_camel_case_props_stay_required() {
    check_pack(
        "required-camel-case-props-3791",
        "fix_history_cases::required_camel_case_props_3791::required_camel_case_props_stay_required",
        "121da88bf450158bd7bccf8f12da8db9c97b77af",
        Some(3791),
        &[
            "omitted-required-camel-props",
            "complete-camel-and-kebab-props",
            "wide-props-missing-required-title",
            "wide-props-complete",
        ],
    );
}
