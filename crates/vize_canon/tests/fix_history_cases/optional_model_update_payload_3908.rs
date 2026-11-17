use crate::support::fix_history_runner::check_pack;

#[test]
fn optional_define_model_update_payload_carries_undefined() {
    check_pack(
        "optional-model-update-payload-3908",
        "fix_history_cases::optional_model_update_payload_3908::optional_define_model_update_payload_carries_undefined",
        "9074dbf00c23d7679ba4ffc16c2ef9f641ac6d42",
        Some(3908),
        &[
            "model-update-handlers-match-payloads",
            "optional-model-rejects-bare-string-handler",
        ],
    );
}
