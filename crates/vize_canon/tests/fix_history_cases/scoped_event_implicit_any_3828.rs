use crate::support::fix_history_runner::check_pack;

#[test]
fn unresolved_ambient_event_callback_keeps_implicit_any_beside_slot_binding() {
    check_pack(
        "scoped-event-implicit-any-3828",
        "fix_history_cases::scoped_event_implicit_any_3828::unresolved_ambient_event_callback_keeps_implicit_any_beside_slot_binding",
        "7b0b4adb66ff112a03ecf4da57641214c01f04d7",
        Some(3828),
        &[
            "unresolved-ambient-event-slot-name-collision-implicit-any",
            "unresolved-ambient-event-annotated-parameter",
        ],
    );
}
