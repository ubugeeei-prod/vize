use crate::support::fix_history_runner::check_pack;

#[test]
fn same_element_v_if_narrows_event_handlers() {
    check_pack(
        "vif-narrowed-event-handler-1142",
        "fix_history_cases::vif_narrowed_event_handler_1142::same_element_v_if_narrows_event_handlers",
        "4e8f5317fe371bb74dbd0731f28f78156db866a5",
        Some(1142),
        &[
            "same-element-vif-narrows-handler",
            "same-element-vif-wrong-branch",
            "reserved-prop-guard-narrows-handler",
            "reserved-prop-guard-wrong-branch",
        ],
    );
}
