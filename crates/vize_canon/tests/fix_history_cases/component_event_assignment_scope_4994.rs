use crate::support::fix_history_runner::check_pack;

#[test]
fn component_event_assignment_does_not_narrow_render_scope() {
    check_pack(
        "component-event-assignment-scope-4994",
        "fix_history_cases::component_event_assignment_scope_4994::component_event_assignment_does_not_narrow_render_scope",
        "dfe3b6befc643c46031356e3b357e73cb1afe1ee",
        Some(4994),
        &[
            "listener-assignment-keeps-sibling-binding-wide",
            "listener-assignment-wrong-value",
            "sibling-comparison-without-overlap",
        ],
    );
}
