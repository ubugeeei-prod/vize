use crate::support::fix_history_runner::check_pack;

#[test]
fn slot_outlet_spreads_keep_authored_payload_members() {
    check_pack(
        "slot-outlet-spread-payloads-6398",
        "fix_history_cases::slot_outlet_spread_payloads_6398::slot_outlet_spreads_keep_authored_payload_members",
        "a75c629c5c948885f7ffe2216f044a1a6c5e2851",
        Some(6398),
        &[
            "spread-nested-clean",
            "spread-nested-incompatible-payload",
            "spread-shadowed-generic-extract-clean",
            "spread-shadowed-generic-extract-incompatible-payload",
            "spread-shadowed-extract-clean",
            "spread-shadowed-extract-incompatible-payload",
            "spread-partial-ordered-optional-generic-payloads",
            "spread-missing-reports-exact-error",
            "spread-optional-reports-exact-error",
            "spread-order-reports-exact-error",
            "spread-callback-reports-exact-error",
            "spread-inline-callback-reports-exact-error",
        ],
    );
}
