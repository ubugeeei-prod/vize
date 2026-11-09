use crate::support::fix_history_runner::check_pack;

#[test]
fn optional_handlers_preserve_batch_diagnostics() {
    check_pack(
        "optional-event-handlers-3148",
        "fix_history_cases::optional_event_handlers_3148::optional_handlers_preserve_batch_diagnostics",
        "805d32a5277a025d062edc47e20cc22ce170f5e7",
        Some(3148),
        &[
            "optional-component-handler",
            "optional-native-handler",
            "optional-component-handler-payload-error",
        ],
    );
}
