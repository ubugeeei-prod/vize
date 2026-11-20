use crate::support::fix_history_runner::check_pack;

#[test]
fn inline_arrow_event_handlers_are_called_with_the_event() {
    check_pack(
        "inline-arrow-event-handler-1141",
        "fix_history_cases::inline_arrow_event_handler_1141::inline_arrow_event_handlers_are_called_with_the_event",
        "f1ba7b90698fc3ef57a6f4fcb2a8b641d5ab6053",
        Some(1141),
        &[
            "inline-arrow-payload-contextually-typed",
            "inline-callback-forms",
            "inline-arrow-payload-wrong-member",
            "inline-arrow-payload-wrong-annotation",
        ],
    );
}
