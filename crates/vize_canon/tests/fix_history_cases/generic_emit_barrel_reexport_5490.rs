use crate::support::fix_history_runner::check_pack;

#[test]
fn reexported_generic_emit_payload_specializes_from_enum_member_prop() {
    check_pack(
        "generic-emit-barrel-reexport-5490",
        "fix_history_cases::generic_emit_barrel_reexport_5490::reexported_generic_emit_payload_specializes_from_enum_member_prop",
        "b31b1e64630621cf00cb2d5cd6189e0cf80acdd9",
        Some(5490),
        &[
            "enum-member-prop-specializes-barrel-emit",
            "mismatched-enum-member-rejects-barrel-listener",
            "published-package-sfc-keeps-ambient-stub",
        ],
    );
}
