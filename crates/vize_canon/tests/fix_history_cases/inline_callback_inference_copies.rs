use crate::support::fix_history_runner::check_pack;

#[test]
fn inference_only_callback_copies_do_not_duplicate_diagnostics() {
    check_pack(
        "inline-callback-inference-copies",
        "fix_history_cases::inline_callback_inference_copies::inference_only_callback_copies_do_not_duplicate_diagnostics",
        "f50661c12a54884715c430c1225f367569ebc05a",
        None,
        &[
            "inline-callback-prop-and-event-contextual",
            "inline-callback-prop-wrong-member-reported-once",
            "inline-event-callback-wrong-member-reported-once",
        ],
    );
}
