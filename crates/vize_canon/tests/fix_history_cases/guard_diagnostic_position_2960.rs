use crate::support::fix_history_runner::check_pack;

#[test]
fn authored_guards_own_batch_diagnostics() {
    check_pack(
        "guard-diagnostic-position-2960",
        "fix_history_cases::guard_diagnostic_position_2960::authored_guards_own_batch_diagnostics",
        "7f01c58cdd13bda478e7b1b70fe5575a173bd063",
        Some(2960),
        &[
            "authored-condition",
            "authored-condition-owns-comparison-error",
        ],
    );
}
