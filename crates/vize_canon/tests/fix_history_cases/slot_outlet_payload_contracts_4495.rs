use crate::support::fix_history_runner::check_pack;

#[test]
fn slot_outlet_payloads_type_destructured_slot_bindings() {
    check_pack(
        "slot-outlet-payload-contracts-4495",
        "fix_history_cases::slot_outlet_payload_contracts_4495::slot_outlet_payloads_type_destructured_slot_bindings",
        "8ab7fd93f0072880f46ea7fce8ac9f998922feed",
        Some(4495),
        &[
            "declared-slot-payloads-clean",
            "unknown-slot-payload-member",
            "omitted-declared-slot",
            "open-slot-index-signature",
            "open-slot-index-signature-wrong-payload-use",
        ],
    );
}
