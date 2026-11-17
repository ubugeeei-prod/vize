use crate::support::fix_history_runner::check_pack;

#[test]
fn template_dollar_emit_consumes_the_define_emits_binding() {
    check_pack(
        "define-emits-template-dollar-emit-3708",
        "fix_history_cases::define_emits_template_dollar_emit_3708::template_dollar_emit_consumes_the_define_emits_binding",
        "34463efe9ac81e18619ebcdc3e3326b94bd85932",
        Some(3708),
        &[
            "dollar-emit-with-unused-local",
            "renamed-binding-dollar-emit",
            "renamed-binding-dollar-emit-wrong-event",
            "direct-template-emit",
            "direct-script-emit",
            "unused-define-emits-result",
            "options-api-unrelated-local",
        ],
    );
}
