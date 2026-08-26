#![allow(clippy::disallowed_types)]

use vize_atelier_core::{ErrorCode, TemplateSyntaxMode};
use vize_atelier_dom::{DomCompilerOptions, compile_template_with_template_syntax};
use vize_s0::{Allocator, String};

fn compile(source: &str, mode: TemplateSyntaxMode) -> (Vec<ErrorCode>, String) {
    let allocator = Allocator::new();
    let options = DomCompilerOptions {
        prefix_identifiers: true,
        ..DomCompilerOptions::default()
    };
    let (_, errors, result) =
        compile_template_with_template_syntax(&allocator, source, options, mode);
    (
        errors.into_iter().map(|error| error.code).collect(),
        result.code,
    )
}

#[test]
fn quirks_preserves_vue2_conditional_component_model_callback() {
    let (errors, code) = compile(
        r#"<el-input v-model="multiple ? presentText : inputValue" />"#,
        TemplateSyntaxMode::Quirks,
    );

    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    assert!(
        code.contains("$event => (_ctx.multiple ? _ctx.presentText : _ctx.inputValue = $event)"),
        "Vue 2 callback shape was not preserved:\n{code}"
    );
    assert!(
        !code.contains("((_ctx.multiple ? _ctx.presentText : _ctx.inputValue) = $event)"),
        "conditional target must not be parenthesized in quirks mode:\n{code}"
    );
}

#[test]
fn quirks_preserves_conditional_native_model_callback() {
    let (errors, code) = compile(
        r#"<input v-model="enabled ? primary : fallback.value" />"#,
        TemplateSyntaxMode::Quirks,
    );

    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    assert!(
        code.contains("$event => (_ctx.enabled ? _ctx.primary : _ctx.fallback.value = $event)"),
        "native model callback did not assign its alternate branch:\n{code}"
    );
}

#[test]
fn standard_and_strict_reject_conditional_model_targets() {
    let source = r#"<el-input v-model="multiple ? presentText : inputValue" />"#;

    for mode in [TemplateSyntaxMode::Standard, TemplateSyntaxMode::Strict] {
        let (errors, _) = compile(source, mode);
        assert_eq!(
            errors,
            [ErrorCode::InvalidExpression],
            "conditional compatibility leaked into {mode:?}"
        );
    }
}

#[test]
fn quirks_does_not_accept_parenthesized_or_binary_model_targets() {
    for (expression, source) in [
        (
            "(multiple ? presentText : inputValue)",
            r#"<el-input v-model="(multiple ? presentText : inputValue)" />"#,
        ),
        ("left + right", r#"<el-input v-model="left + right" />"#),
    ] {
        let (errors, _) = compile(source, TemplateSyntaxMode::Quirks);
        assert_eq!(
            errors,
            [ErrorCode::InvalidExpression],
            "non-compatible target {expression:?} was accepted"
        );
    }
}
