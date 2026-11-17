use crate::support::fix_history_runner::check_pack;

#[test]
fn template_key_expressions_follow_the_vue_key_contract() {
    check_pack(
        "template-key-expression-typing-4240",
        "fix_history_cases::template_key_expression_typing_4240::template_key_expressions_follow_the_vue_key_contract",
        "22c638a00bf228aeea0e1b0fe2b9fa70b876a261",
        Some(4240),
        &[
            "key-expression-members-valid",
            "key-expression-members-checked-inside-fragment",
            "template-fragment-keys",
            "element-keys-reserved-prop-contract",
            "component-keys-outside-props-and-events",
        ],
    );
}
