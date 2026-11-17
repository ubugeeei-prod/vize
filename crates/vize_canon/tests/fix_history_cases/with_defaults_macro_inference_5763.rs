use crate::support::fix_history_runner::check_pack;

#[test]
fn with_defaults_macro_inference_matches_vue_readonly_semantics() {
    check_pack(
        "with-defaults-macro-inference-5763",
        "fix_history_cases::with_defaults_macro_inference_5763::with_defaults_macro_inference_matches_vue_readonly_semantics",
        "91754f691b70884a7b1723fb723388485b2ed54b",
        Some(5763),
        &[
            "reka-generic-vmodel-expect-error-consumed",
            "reka-generic-vmodel-unsuppressed",
            "template-props-default-resolved",
            "template-undefaulted-prop-read",
            "direct-template-prop-narrowed",
            "never-prop-not-boolean",
            "with-defaults-props-readonly",
        ],
    );
}
