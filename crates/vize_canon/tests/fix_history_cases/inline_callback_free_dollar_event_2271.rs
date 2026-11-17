use crate::support::fix_history_runner::check_pack;

#[test]
fn inline_arrow_listener_keeps_authored_free_dollar_event_free() {
    check_pack(
        "inline-callback-free-dollar-event-2271",
        "fix_history_cases::inline_callback_free_dollar_event_2271::inline_arrow_listener_keeps_authored_free_dollar_event_free",
        "3b23363761388106de0f79a7526bc389de92214c",
        Some(2271),
        &[
            "arrow-callback-declared-parameter",
            "inline-statement-implicit-event",
            "arrow-callback-free-dollar-event",
            "arrow-callback-wrong-argument",
        ],
    );
}
