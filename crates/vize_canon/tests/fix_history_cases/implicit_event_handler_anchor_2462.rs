use crate::support::fix_history_runner::check_pack;

#[test]
fn implicit_event_handler_reference_maps_to_the_authored_expression() {
    check_pack(
        "implicit-event-handler-anchor-2462",
        "fix_history_cases::implicit_event_handler_anchor_2462::implicit_event_handler_reference_maps_to_the_authored_expression",
        "b34acb9e2a3a27ea3b4981c41ed64c42367447c2",
        Some(2462),
        &[
            "implicit-handler-accepts-event",
            "implicit-handler-parameter-mismatch",
        ],
    );
}
