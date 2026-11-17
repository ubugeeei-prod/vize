use crate::support::fix_history_runner::check_pack;

#[test]
fn dynamic_member_component_listener_infers_emit_payload() {
    check_pack(
        "dynamic-member-component-emits-2915",
        "fix_history_cases::dynamic_member_component_emits_2915::dynamic_member_component_listener_infers_emit_payload",
        "9c8861cfcbca6ba118dfff9b14250bb30e6aee39",
        Some(2915),
        &[
            "dynamic-member-listener-payload",
            "dynamic-member-listener-payload-mismatch",
        ],
    );
}
