use crate::support::fix_history_runner::check_pack;

#[test]
fn undefined_unknown_component_listener_is_optional() {
    check_pack(
        "undefined-unknown-listener-2956",
        "fix_history_cases::undefined_unknown_listener_2956::undefined_unknown_component_listener_is_optional",
        "872d43f244876bf2b3241026abe3f1e2a552d58a",
        Some(2956),
        &[
            "undefined-unknown-listener",
            "non-callable-unknown-listener",
            "unresolved-unknown-listener",
        ],
    );
}
