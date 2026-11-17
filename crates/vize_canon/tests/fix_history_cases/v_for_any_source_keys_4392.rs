use crate::support::fix_history_runner::check_pack;

#[test]
fn any_v_for_key_matches_vue_tsc_object_fallback() {
    check_pack(
        "v-for-any-source-keys-4392",
        "fix_history_cases::v_for_any_source_keys_4392::any_v_for_key_matches_vue_tsc_object_fallback",
        "8ede47a7ddf0eb07a9a19b60522156bd378a03ff",
        Some(4392),
        &[
            "any-source-key-converted-before-comparison",
            "any-source-key-compared-to-number",
        ],
    );
}
