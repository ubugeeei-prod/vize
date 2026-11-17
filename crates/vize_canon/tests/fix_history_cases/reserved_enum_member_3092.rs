use crate::support::fix_history_runner::check_pack;

#[test]
fn reserved_enum_members_preserve_batch_diagnostics() {
    check_pack(
        "reserved-enum-member-3092",
        "fix_history_cases::reserved_enum_member_3092::reserved_enum_members_preserve_batch_diagnostics",
        "347d653aba6f5bc24592275a772e0877f367ce05",
        Some(3092),
        &["reserved-enum-members", "reserved-enum-member-type-error"],
    );
}
