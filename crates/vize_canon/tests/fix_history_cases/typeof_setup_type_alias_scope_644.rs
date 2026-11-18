use crate::support::fix_history_runner::check_pack;

#[test]
fn typeof_anchored_type_alias_stays_in_setup_scope() {
    check_pack(
        "typeof-setup-type-alias-scope-644",
        "fix_history_cases::typeof_setup_type_alias_scope_644::typeof_anchored_type_alias_stays_in_setup_scope",
        "07574f1faea7dec3be8f3177851a1a45c6c0c937",
        Some(644),
        &[
            "typeof-setup-const-type-alias",
            "typeof-setup-const-type-alias-wrong-literal",
        ],
    );
}
