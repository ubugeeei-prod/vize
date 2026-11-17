use crate::support::fix_history_runner::check_pack;

#[test]
fn setup_local_type_aliases_stay_with_their_consts() {
    check_pack(
        "setup-derived-type-aliases-2037",
        "fix_history_cases::setup_derived_type_aliases_2037::setup_local_type_aliases_stay_with_their_consts",
        "e9bfbcc91101021a20a5496ebdcf1740c99e7539",
        Some(2037),
        &[
            "transitive-typeof-alias-props",
            "transitive-typeof-alias-wrong-state",
            "local-alias-before-ref-usage",
            "local-alias-ref-wrong-literal",
        ],
    );
}
