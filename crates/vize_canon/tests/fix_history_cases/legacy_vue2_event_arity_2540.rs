use crate::support::fix_history_runner::check_pack;

#[test]
fn legacy_vue2_unresolved_component_event_payloads_stay_variadic() {
    check_pack(
        "legacy-vue2-event-arity-2540",
        "fix_history_cases::legacy_vue2_event_arity_2540::legacy_vue2_unresolved_component_event_payloads_stay_variadic",
        "1d9cbd29df81c9953c6f28e95eb829b92e834960",
        Some(2540),
        &["unresolved-and-empty-event-listeners-stay-variadic"],
    );
}
