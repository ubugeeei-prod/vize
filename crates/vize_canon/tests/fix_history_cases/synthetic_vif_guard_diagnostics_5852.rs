use crate::support::fix_history_runner::check_pack;

#[test]
fn component_vif_guard_callback_reports_only_from_authored_guard() {
    check_pack(
        "synthetic-vif-guard-diagnostics-5852",
        "fix_history_cases::synthetic_vif_guard_diagnostics_5852::component_vif_guard_callback_reports_only_from_authored_guard",
        "0726ec9cee5660f87b79748be715bf970c3190e8",
        Some(5852),
        &[
            "typed-guard-callback",
            "implicit-any-guard-callback-reported-once",
            "typed-guard-wrong-prop-reported-once",
        ],
    );
}
