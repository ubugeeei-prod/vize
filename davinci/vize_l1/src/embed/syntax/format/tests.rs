use oxc_allocator::Allocator;
use oxc_ast::ast::{Expression, Statement};
use oxc_span::SourceType;

use super::{FORMAT_PARSE_OPTIONS, format_source_type, parse_program_for_format};

fn single_expression<'p, 'a>(statements: &'p [Statement<'a>]) -> &'p Expression<'a> {
    let [Statement::ExpressionStatement(statement)] = statements else {
        panic!("one expression statement");
    };
    &statement.expression
}

#[test]
fn profile_drops_parentheses_and_skips_regex_and_identifier_hashes() {
    assert!(!FORMAT_PARSE_OPTIONS.preserve_parens);
    assert!(!FORMAT_PARSE_OPTIONS.parse_regular_expression);
    assert!(!FORMAT_PARSE_OPTIONS.enable_ident_hashes);
    assert!(FORMAT_PARSE_OPTIONS.allow_return_outside_function);
    assert!(FORMAT_PARSE_OPTIONS.allow_v8_intrinsics);

    let allocator = Allocator::default();
    let parsed = parse_program_for_format(&allocator, "void ((a + b));", SourceType::mjs());
    assert!(parsed.diagnostics.is_empty());
    let program = &parsed.program;
    let Expression::UnaryExpression(unary) = single_expression(&program.body) else {
        panic!("void expression");
    };
    assert!(matches!(unary.argument, Expression::BinaryExpression(_)));
}

#[test]
fn javascript_always_admits_jsx_and_typescript_keeps_its_variant() {
    for source_type in [SourceType::mjs(), SourceType::cjs(), SourceType::jsx()] {
        assert!(format_source_type(source_type).is_jsx());
        let allocator = Allocator::default();
        let parsed = parse_program_for_format(&allocator, "<a>{b}</a>;", source_type);
        assert!(parsed.diagnostics.is_empty());
    }
    assert!(!format_source_type(SourceType::ts()).is_jsx());
    assert!(format_source_type(SourceType::tsx()).is_jsx());

    let allocator = Allocator::default();
    let assertion = parse_program_for_format(&allocator, "<T>value;", SourceType::ts());
    assert!(assertion.diagnostics.is_empty());
    let program = &assertion.program;
    assert!(matches!(
        single_expression(&program.body),
        Expression::TSTypeAssertion(_)
    ));
}

#[test]
fn diagnostics_are_retained_for_the_printer_to_refuse() {
    let allocator = Allocator::default();
    let returned = parse_program_for_format(&allocator, "return value;", SourceType::mjs());
    assert!(returned.diagnostics.is_empty());

    let broken = parse_program_for_format(&allocator, "const = ;", SourceType::ts());
    assert!(!broken.diagnostics.is_empty());
}
