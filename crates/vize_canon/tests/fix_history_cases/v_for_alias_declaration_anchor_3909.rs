use crate::support::fix_history_runner::check_pack;

#[test]
fn v_for_alias_declarations_anchor_pattern_diagnostics() {
    check_pack(
        "v-for-alias-declaration-anchor-3909",
        "fix_history_cases::v_for_alias_declaration_anchor_3909::v_for_alias_declarations_anchor_pattern_diagnostics",
        "31eecf0427b56b8221290e16ec73022d73633f9b",
        Some(3909),
        &[
            "alias-declarations",
            "missing-destructured-alias-property-invalid",
        ],
    );
}
