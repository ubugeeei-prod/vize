use crate::support::fix_history_runner::check_pack;

#[test]
fn async_setup_follows_top_level_await_in_the_script_ast() {
    check_pack(
        "ast-async-setup-1432",
        "fix_history_cases::ast_async_setup_1432::async_setup_follows_top_level_await_in_the_script_ast",
        "e8ad806442134436b0a2750d6a93b63b627dceb1",
        Some(1432),
        &[
            "top-level-await",
            "top-level-await-wrong-member",
            "top-level-for-await",
            "top-level-for-await-wrong-member",
            "nested-and-text-await",
            "nested-await-wrong-annotation",
        ],
    );
}
