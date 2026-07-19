//! Snapshot coverage for `<slot>` outlet lowering in the Vapor compiler.
//!
//! Kept separate from `tests.rs` so that already large file does not grow past
//! the source-file-length limit; the small assertion helpers mirror the ones
//! defined there.

use super::compile_vapor;
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_carton::Bump;

fn normalize_code(code: &str) -> String {
    code.lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

fn assert_parses_as_module(code: &str) {
    let allocator = Allocator::default();
    let parsed = Parser::new(
        &allocator,
        code,
        SourceType::default()
            .with_module(true)
            .with_typescript(true),
    )
    .parse();

    assert!(
        parsed.errors.is_empty(),
        "generated code should parse, got: {:?}\n\n{}",
        parsed.errors,
        code
    );
}

#[test]
fn test_compile_bare_default_slot_outlet() {
    let allocator = Bump::new();
    let result = compile_vapor(&allocator, r#"<slot />"#, Default::default());

    assert!(
        result.error_messages.is_empty(),
        "Expected no errors: {:?}",
        result.error_messages
    );

    let code = normalize_code(&result.code);
    assert_parses_as_module(&code);
    insta::assert_snapshot!(code.as_str());
}

#[test]
fn test_compile_named_slot_outlet_without_props() {
    let allocator = Bump::new();
    let result = compile_vapor(&allocator, r#"<slot name="head" />"#, Default::default());

    assert!(
        result.error_messages.is_empty(),
        "Expected no errors: {:?}",
        result.error_messages
    );

    let code = normalize_code(&result.code);
    assert_parses_as_module(&code);
    insta::assert_snapshot!(code.as_str());
}

#[test]
fn test_compile_default_slot_outlet_with_fallback_only() {
    let allocator = Bump::new();
    let result = compile_vapor(
        &allocator,
        r#"<slot><div>fb</div></slot>"#,
        Default::default(),
    );

    assert!(
        result.error_messages.is_empty(),
        "Expected no errors: {:?}",
        result.error_messages
    );

    let code = normalize_code(&result.code);
    assert_parses_as_module(&code);
    insta::assert_snapshot!(code.as_str());
}

#[test]
fn test_compile_slot_outlet_static_and_spread_props() {
    let allocator = Bump::new();
    let result = compile_vapor(
        &allocator,
        r#"<slot kind="primary" :row="item" v-bind="extra" />"#,
        Default::default(),
    );

    assert!(
        result.error_messages.is_empty(),
        "Expected no errors: {:?}",
        result.error_messages
    );

    let code = normalize_code(&result.code);
    assert_parses_as_module(&code);
    insta::assert_snapshot!(code.as_str());
}
