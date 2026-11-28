use crate::support::fix_history_runner::check_pack;

#[test]
fn options_api_extends_base_exposes_inherited_template_names() {
    check_pack(
        "options-extends-inherited-names-2134",
        "fix_history_cases::options_extends_inherited_names_2134::options_api_extends_base_exposes_inherited_template_names",
        "bd113bbccb2be992217f99d60b620f3715e4f1b5",
        Some(2134),
        &[
            "extends-base-template-names",
            "extends-base-method-argument-checked",
            "define-component-extends-base-members-checked",
        ],
    );
}
