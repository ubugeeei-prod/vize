use crate::support::fix_history_runner::check_pack;

#[test]
fn v_for_callbacks_preserve_batch_diagnostics() {
    check_pack(
        "template-loop-narrowing-3149",
        "fix_history_cases::template_loop_narrowing_3149::v_for_callbacks_preserve_batch_diagnostics",
        "2ecf10de92663303cecb76470bfa5ea42e992764",
        Some(3149),
        &[
            "v-for-callback-outer-discriminant",
            "v-for-callback-prop-error",
        ],
    );
}
