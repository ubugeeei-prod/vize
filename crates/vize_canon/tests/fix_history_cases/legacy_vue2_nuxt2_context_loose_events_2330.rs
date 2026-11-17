use crate::support::fix_history_runner::check_pack;

#[test]
fn legacy_vue2_keeps_nuxt2_page_context_and_loose_object_event_payloads() {
    check_pack(
        "legacy-vue2-nuxt2-context-loose-events-2330",
        "fix_history_cases::legacy_vue2_nuxt2_context_loose_events_2330::legacy_vue2_keeps_nuxt2_page_context_and_loose_object_event_payloads",
        "246c6c0678c3fddd27b66f3bf1a630d684d8cd03",
        Some(2330),
        &[
            "nuxt2-page-validate-context",
            "nuxt2-page-validate-params-wrong-member",
            "object-event-payload-stays-loose",
            "matched-object-event-payload-stays-loose",
            "matched-number-event-payload-still-checked",
        ],
    );
}
