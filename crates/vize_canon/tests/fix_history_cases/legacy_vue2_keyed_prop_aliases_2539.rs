use crate::support::fix_history_runner::check_pack;

#[test]
fn legacy_vue2_unresolved_keyed_props_use_loose_record_fallback() {
    check_pack(
        "legacy-vue2-keyed-prop-aliases-2539",
        "fix_history_cases::legacy_vue2_keyed_prop_aliases_2539::legacy_vue2_unresolved_keyed_props_use_loose_record_fallback",
        "3d2e641317a3476472dcbe72c0dce8283ebe1d97",
        Some(2539),
        &[
            "vuetify-style-unknown-prop-aliases",
            "declared-optional-prop-misuse",
        ],
    );
}
