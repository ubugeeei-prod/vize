use crate::support::fix_history_runner::check_pack;

#[test]
fn fragment_keys_and_external_member_listeners_match_vue_tsc() {
    check_pack(
        "listener-key-vue-tsc-parity-5960",
        "fix_history_cases::listener_key_vue_tsc_parity_5960::fragment_keys_and_external_member_listeners_match_vue_tsc",
        "e30b36c8f3302aa9fc364bc288f44ac9e732ff65",
        Some(5960),
        &[
            "fragment-property-keys-and-unchecked-id",
            "fragment-keys-checked-against-property-key",
            "external-member-listener-exact-payload",
            "external-member-listener-strict-contract",
        ],
    );
}
