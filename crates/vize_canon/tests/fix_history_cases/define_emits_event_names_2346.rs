use crate::support::fix_history_runner::check_pack;

#[test]
fn define_emits_callable_infers_event_names() {
    check_pack(
        "define-emits-event-names-2346",
        "fix_history_cases::define_emits_event_names_2346::define_emits_callable_infers_event_names",
        "6096c8bca193e3a73fb28918ca78613516ec0a02",
        Some(2346),
        &[
            "emit-passed-to-event-name-helper",
            "emit-wrong-payload",
            "runtime-emit-object-instance-props",
            "runtime-emit-object-template-listener",
            "runtime-emit-object-wrong-listener",
        ],
    );
}
