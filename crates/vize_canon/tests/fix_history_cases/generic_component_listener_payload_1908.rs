use crate::support::fix_history_runner::check_pack;

#[test]
fn generic_component_listener_payload_follows_inferred_props() {
    check_pack(
        "generic-component-listener-payload-1908",
        "fix_history_cases::generic_component_listener_payload_1908::generic_component_listener_payload_follows_inferred_props",
        "2105341f2a79f0801a383a49740566ba03fcbeee",
        Some(1908),
        &[
            "compatible-generic-listener",
            "historical-compatible-and-incompatible-listeners",
        ],
    );
}
