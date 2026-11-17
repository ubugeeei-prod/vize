use crate::support::fix_history_runner::check_pack;

#[test]
fn setup_local_type_alias_stays_visible_to_hoisted_props() {
    check_pack(
        "setup-local-type-alias-props-1946",
        "fix_history_cases::setup_local_type_alias_props_1946::setup_local_type_alias_stays_visible_to_hoisted_props",
        "ce15831f96c2963db2f56740f3edf45bf18665dd",
        Some(1946),
        &[
            "local-alias-prop-accepts-member",
            "local-alias-prop-rejects-non-member",
        ],
    );
}
