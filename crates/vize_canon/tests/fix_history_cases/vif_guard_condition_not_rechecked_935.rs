use crate::support::fix_history_runner::check_pack;

#[test]
fn vif_guard_condition_is_not_rechecked_inside_own_guard() {
    check_pack(
        "vif-guard-condition-not-rechecked-935",
        "fix_history_cases::vif_guard_condition_not_rechecked_935::vif_guard_condition_is_not_rechecked_inside_own_guard",
        "bc0e6f324f11239797c25a07e966f41e2c2dd4fc",
        Some(935),
        &[
            "vif-condition-not-rechecked-in-own-guard",
            "nested-vif-compares-excluded-literal",
        ],
    );
}
