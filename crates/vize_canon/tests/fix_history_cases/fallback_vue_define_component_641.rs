use crate::support::fix_history_runner::check_pack;

#[test]
fn fallback_vue_stub_exports_define_component() {
    check_pack(
        "fallback-vue-define-component-641",
        "fix_history_cases::fallback_vue_define_component_641::fallback_vue_stub_exports_define_component",
        "93a5c14e4032bbf361b5b989a497b2b16c13139c",
        Some(641),
        &[
            "stubbed-vue-define-component-import",
            "stubbed-vue-define-component-extra-argument",
        ],
    );
}
