use crate::support::fix_history_runner::check_pack;

#[test]
fn external_component_slot_payloads_keep_their_types() {
    check_pack(
        "external-slot-payload-types-4244",
        "fix_history_cases::external_slot_payload_types_4244::external_component_slot_payloads_keep_their_types",
        "4edbc20c6e588e5ebb64bd80e4ba9ec51af56e08",
        Some(4244),
        &[
            "generic-child-inferable-slot-payloads",
            "generic-child-slot-payload-fallbacks",
            "declared-and-package-child-slot-payloads",
            "declared-and-package-child-slot-payload-misuse",
            "v-for-binding-types-unchanged",
        ],
    );
}
