//! Diagnostics for template expressions that fail to parse.
//!
//! The official compiler (`@vue/compiler-core` with `prefixIdentifiers`)
//! reports `X_INVALID_EXPRESSION` ("Error parsing JavaScript expression: …")
//! when a template expression cannot be parsed. Vize previously passed the
//! raw content through with no diagnostic (#1394). These tests pin the new
//! behavior: exactly one diagnostic per invalid expression, carrying the
//! expression's source span, and no diagnostics (or output changes) for
//! valid expressions.
#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

use vize_atelier_core::{CompilerError, ErrorCode};
use vize_atelier_dom::{DomCompilerOptions, compile_template_with_options};
use vize_carton::Bump;

fn analyzed_options() -> DomCompilerOptions {
    DomCompilerOptions {
        // The analyzed (SFC) lane always prefixes identifiers; this is
        // the mode where vue-core parses expressions and reports errors.
        prefix_identifiers: true,
        ..DomCompilerOptions::default()
    }
}

fn compile(source: &str) -> (Vec<CompilerError>, String) {
    let allocator = Bump::new();
    let (_, errors, result) = compile_template_with_options(&allocator, source, analyzed_options());
    (errors, result.code.to_string())
}

fn assert_single_invalid_expression(source: &str, expression: &str) {
    let (errors, _) = compile(source);
    assert_eq!(
        errors.len(),
        1,
        "expected exactly one diagnostic for {source:?}, got {errors:?}"
    );
    let error = &errors[0];
    assert_eq!(error.code, ErrorCode::InvalidExpression, "in {source:?}");
    assert!(
        error
            .message
            .starts_with("Error parsing JavaScript expression: "),
        "unexpected message {:?}",
        error.message
    );

    let loc = error
        .loc
        .as_ref()
        .unwrap_or_else(|| panic!("diagnostic for {source:?} should carry a span"));
    assert_eq!(
        loc.source.as_str(),
        expression,
        "span text mismatch in {source:?}"
    );
    let expected_offset = source
        .find(expression)
        .expect("expression must appear in template") as u32;
    assert_eq!(
        loc.start.offset, expected_offset,
        "span start offset mismatch in {source:?}"
    );
    assert_eq!(
        loc.end.offset,
        expected_offset + expression.len() as u32,
        "span end offset mismatch in {source:?}"
    );
}

#[test]
fn invalid_interpolation_expression_reports_diagnostic() {
    assert_single_invalid_expression("<div>{{ foo( }}</div>", "foo(");
}

#[test]
fn invalid_v_if_expression_reports_diagnostic() {
    assert_single_invalid_expression(r#"<div v-if="foo(">x</div>"#, "foo(");
}

#[test]
fn invalid_v_for_source_reports_diagnostic() {
    assert_single_invalid_expression(r#"<div v-for="item in items((">x</div>"#, "items((");
}

#[test]
fn invalid_event_handler_reports_diagnostic() {
    assert_single_invalid_expression(r#"<button @click="count +">x</button>"#, "count +");
}

#[test]
fn valid_expressions_compile_without_diagnostics() {
    let templates = [
        "<div>{{ foo() }}</div>",
        r#"<div v-if="ok && items.length">x</div>"#,
        r#"<div v-for="(item, i) in items">{{ item }}</div>"#,
        r#"<button @click="count++">x</button>"#,
        // Multi-statement handler: parses as a program, not an expression.
        r#"<button @click="a++; b++">x</button>"#,
        // Reserved words are valid bindings via the simple-identifier path.
        "<div>{{ class }}</div>",
    ];
    for template in templates {
        let (errors, code) = compile(template);
        assert!(
            errors.is_empty(),
            "expected no diagnostics for {template:?}, got {errors:?}"
        );
        assert!(!code.is_empty(), "expected render code for {template:?}");
    }
}

#[test]
fn success_path_output_is_unchanged() {
    // The diagnostic path only runs on parse failure; a valid template must
    // produce byte-identical output with zero collected errors.
    let allocator = Bump::new();
    let source = r#"<div v-if="show" v-for="item in list" @click="go(item)">{{ item.name }}</div>"#;
    let (_, errors, result) = compile_template_with_options(&allocator, source, analyzed_options());
    assert!(errors.is_empty(), "unexpected diagnostics: {errors:?}");
    insta::assert_snapshot!(result.code.as_str());
}
