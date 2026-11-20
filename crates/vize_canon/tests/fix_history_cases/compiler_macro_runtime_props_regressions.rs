use crate::support::fix_history_runner::check_pack;

#[test]
fn compiler_macro_runtime_props_and_shadowing_regressions() {
    check_pack(
        "compiler-macro-runtime-props-regressions",
        "fix_history_cases::compiler_macro_runtime_props_regressions::compiler_macro_runtime_props_and_shadowing_regressions",
        "79e9e110ba725255624cea545bcd6dd657e78348",
        None,
        &[
            "runtime-declared-props",
            "runtime-declared-props-errors",
            "props-destructure-template-shadow",
            "props-destructure-wrong-member",
            "emits-method-interface",
            "emits-method-interface-errors",
            "provide-inject-symbol",
            "provide-inject-symbol-error",
            "top-level-await",
            "non-scoped-style-block",
            "nested-ref-value-component-prop",
            "nested-ref-value-component-prop-string",
        ],
    );
}
