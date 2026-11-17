use crate::support::fix_history_runner::check_pack;

#[test]
fn vfor_source_is_narrowed_by_enclosing_vif() {
    check_pack(
        "vfor-source-enclosing-vif-narrowing-1511",
        "fix_history_cases::vfor_source_enclosing_vif_narrowing_1511::vfor_source_is_narrowed_by_enclosing_vif",
        "edd9fbe7bf4a08d82f1a9229dd0fc6a6780f0c56",
        Some(1511),
        &[
            "vfor-source-narrowed-by-enclosing-vif",
            "vfor-source-not-narrowed-without-vif",
            "vif-inside-vfor-body-does-not-wrap-loop",
        ],
    );
}
