use crate::support::fix_history_runner::check_pack;

#[test]
fn asi_separated_event_handler_statements_stay_authored() {
    check_pack(
        "asi-separated-event-handlers-3703",
        "fix_history_cases::asi_separated_event_handlers_3703::asi_separated_event_handler_statements_stay_authored",
        "4039c4c9a5217b81591d67c8123981eba1126f33",
        Some(3703),
        &[
            "asi-separated-emit-statements",
            "asi-separated-second-statement-unknown-event",
            "expression-shaped-handlers",
        ],
    );
}
