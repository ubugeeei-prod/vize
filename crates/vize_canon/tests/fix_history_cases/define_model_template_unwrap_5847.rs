use crate::support::fix_history_runner::check_pack;

#[test]
fn define_model_record_indexes_with_template_v_for_keys() {
    check_pack(
        "define-model-template-unwrap-5847",
        "fix_history_cases::define_model_template_unwrap_5847::define_model_record_indexes_with_template_v_for_keys",
        "00b6fd3939ed3f1c77e1a342215849644f64aded",
        Some(5847),
        &[
            "define-model-record-v-for-index",
            "define-model-number-record-string-assignment",
        ],
    );
}
