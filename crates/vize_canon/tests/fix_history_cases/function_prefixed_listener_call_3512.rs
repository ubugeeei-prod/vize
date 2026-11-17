use crate::support::fix_history_runner::check_pack;

#[test]
fn function_prefixed_listener_call_is_not_an_inline_function() {
    check_pack(
        "function-prefixed-listener-call-3512",
        "fix_history_cases::function_prefixed_listener_call_3512::function_prefixed_listener_call_is_not_an_inline_function",
        "7676592d51c264565259de273ccea511601d25b9",
        Some(3512),
        &[
            "unresolved-component-multi-argument-handler",
            "resolved-component-exact-handler-tuple",
            "resolved-component-wrong-first-argument",
            "function-prefixed-call-handler",
            "function-prefixed-call-wrong-argument",
        ],
    );
}
