use crate::support::fix_history_runner::check_pack;

#[test]
fn check_option_probe_accepts_ts_extensions_without_authored_no_emit() {
    check_pack(
        "check-option-probe-mode-13300",
        "fix_history_cases::check_option_probe_mode_13300::check_option_probe_accepts_ts_extensions_without_authored_no_emit",
        "65a280aca9684d4577ad04ef75da8da05ad235ff",
        Some(13300),
        &[
            "check-mode-valid",
            "check-mode-invalid-consumer",
            "authored-declaration-options",
        ],
    );
}
