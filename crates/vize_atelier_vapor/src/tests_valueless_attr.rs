//! Regression tests for valueless static attributes on components.

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

// Regression tests: a valueless static attribute on a component is an empty
// string prop (matching vdom), not `undefined`, which would drop the
// attribute from the rendered element.
#[test]
fn test_compile_component_valueless_static_attr_is_empty_string_prop() {
    let allocator = Bump::new();
    let result = compile_vapor(&allocator, r#"<MyComp data-probe />"#, Default::default());

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
fn test_compile_dynamic_component_valueless_static_attr_is_empty_string_prop() {
    let allocator = Bump::new();
    let result = compile_vapor(
        &allocator,
        r#"<component :is="tag" data-probe>hi</component>"#,
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
