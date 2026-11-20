use crate::support::fix_history_runner::check_pack;

#[test]
fn component_event_listeners_check_the_full_emit_tuple() {
    check_pack(
        "component-event-full-tuple-1512",
        "fix_history_cases::component_event_full_tuple_1512::component_event_listeners_check_the_full_emit_tuple",
        "4d95a2ddda5f33550cd408b5fa759e80bf9b68d4",
        Some(1512),
        &[
            "tuple-valid",
            "second-parameter-invalid",
            "first-parameter-invalid",
            "native-single-event-valid",
            "native-single-event-invalid",
        ],
    );
}
