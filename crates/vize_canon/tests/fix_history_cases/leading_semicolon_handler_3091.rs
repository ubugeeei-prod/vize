use crate::support::fix_history_runner::check_pack;

#[test]
fn asi_prefixed_handlers_preserve_batch_diagnostics() {
    check_pack(
        "leading-semicolon-handler-3091",
        "fix_history_cases::leading_semicolon_handler_3091::asi_prefixed_handlers_preserve_batch_diagnostics",
        "f6ffab4e808a96f2d0dac8827d4e00c7d7df260a",
        Some(3091),
        &[
            "asi-prefixed-handlers",
            "asi-prefixed-handler-argument-error",
        ],
    );
}
