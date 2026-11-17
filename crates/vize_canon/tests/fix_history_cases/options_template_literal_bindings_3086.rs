use crate::support::fix_history_runner::check_pack;

#[test]
fn options_api_template_literals_are_not_bindings() {
    check_pack(
        "options-template-literal-bindings-3086",
        "fix_history_cases::options_template_literal_bindings_3086::options_api_template_literals_are_not_bindings",
        "c40cb562b37a1cde3080bb560df6fbb23a574212",
        Some(3086),
        &[
            "unresolved-extends-template-literals",
            "unresolved-extends-literal-member-error",
            "setup-spread-template-literals",
            "setup-spread-literal-member-error",
        ],
    );
}
