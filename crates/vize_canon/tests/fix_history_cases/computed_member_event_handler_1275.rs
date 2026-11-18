use crate::support::fix_history_runner::check_pack;

#[test]
fn computed_member_event_handler_references_are_called_with_event() {
    check_pack(
        "computed-member-event-handler-1275",
        "fix_history_cases::computed_member_event_handler_1275::computed_member_event_handler_references_are_called_with_event",
        "b28c78952a0022c512c77ba3ce09273b4db7bd95",
        Some(1275),
        &[
            "computed-member-non-callable-handler",
            "index-member-callable-handler",
            "lexical-member-forms-callable",
            "lexical-member-forms-wrong-payload",
        ],
    );
}
