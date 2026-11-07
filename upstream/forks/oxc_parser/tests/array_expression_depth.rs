use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;

const MAX_ARRAY_EXPRESSION_DEPTH: usize = 256;

fn nested_array(depth: usize) -> String {
    "[".repeat(depth) + "0" + &"]".repeat(depth)
}

#[test]
fn accepts_array_expression_at_the_recursion_limit() {
    let allocator = Allocator::default();
    let source = nested_array(MAX_ARRAY_EXPRESSION_DEPTH);
    let parsed = Parser::new(&allocator, &source, SourceType::ts()).parse_expression();
    assert!(parsed.is_ok(), "the last admitted array depth must remain valid");
}

#[test]
fn rejects_array_expression_beyond_the_recursion_limit() {
    let allocator = Allocator::default();
    let source = nested_array(MAX_ARRAY_EXPRESSION_DEPTH + 1);
    let parsed = Parser::new(&allocator, &source, SourceType::ts()).parse_expression();
    assert!(parsed.is_err(), "untrusted nesting must terminate with a diagnostic");
}
