use crate::support::fix_history_runner::check_pack;

#[test]
fn repeated_slots_unknown_events_and_kebab_props_check_cleanly() {
    check_pack(
        "component-slot-event-kebab-false-positives-2134",
        "fix_history_cases::component_slot_event_kebab_false_positives_2134::repeated_slots_unknown_events_and_kebab_props_check_cleanly",
        "bd113bbccb2be992217f99d60b620f3715e4f1b5",
        Some(2134),
        &[
            "repeated-default-slots",
            "repeated-default-slot-wrong-member",
            "undeclared-component-event-listener",
            "tsx-kebab-prop-alias",
            "tsx-kebab-prop-wrong-type",
        ],
    );
}
