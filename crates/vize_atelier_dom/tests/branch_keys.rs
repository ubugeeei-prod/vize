use vize_atelier_core::{ErrorCode, TemplateSyntaxMode};
use vize_atelier_dom::{DomCompilerOptions, compile_template_with_template_syntax};
use vize_s0::Allocator;

fn compile_errors(source: &str, syntax: TemplateSyntaxMode) -> Vec<ErrorCode> {
    let allocator = Allocator::new();
    let (_, errors, _) = compile_template_with_template_syntax(
        &allocator,
        source,
        DomCompilerOptions::default(),
        syntax,
    );
    errors.into_iter().map(|error| error.code).collect()
}

#[test]
fn quirks_allows_runtime_branch_key_expressions() {
    let errors = compile_errors(
        r#"<div v-if="ok" :key="effect"/><div v-else :key="effect"/>"#,
        TemplateSyntaxMode::Quirks,
    );

    assert!(!errors.contains(&ErrorCode::VIfSameKey), "{errors:?}");
}

#[test]
fn quirks_rejects_duplicate_literal_branch_keys() {
    for source in [
        r#"<div v-if="ok" key="same"/><div v-else key="same"/>"#,
        r#"<div v-if="ok" :key="1"/><div v-else :key="1"/>"#,
    ] {
        let errors = compile_errors(source, TemplateSyntaxMode::Quirks);
        assert!(errors.contains(&ErrorCode::VIfSameKey), "{errors:?}");
    }
}

#[test]
fn standard_keeps_vue_three_dynamic_branch_key_check() {
    let errors = compile_errors(
        r#"<div v-if="ok" :key="effect"/><div v-else :key="effect"/>"#,
        TemplateSyntaxMode::Standard,
    );

    assert!(errors.contains(&ErrorCode::VIfSameKey), "{errors:?}");
}
