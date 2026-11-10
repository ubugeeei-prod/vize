//! A `>` followed by a prefix `!` is relational, so the next `/` opens a regex.
use oxc_allocator::Allocator;
use oxc_ast::ast::{BinaryOperator, Expression, UnaryOperator};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_atelier_core::steps::expression::{
    MAX_EXPRESSION_NESTING_DEPTH, expression_is_safe_to_parse, expression_nesting_depth,
};

const REPRODUCER: &str =
    include_str!("../../../tests/fuzz/regressions/js_ts_expression/issue-12772.input");

fn paren_run_behind(prefix: &str) -> Box<str> {
    let depth = MAX_EXPRESSION_NESTING_DEPTH + 1;
    [prefix, &"(".repeat(depth), "0", &")".repeat(depth), "`"]
        .concat()
        .into_boxed_str()
}

#[test]
fn scheduled_input_is_rejected_before_recursive_expression_parsing() {
    assert_eq!(REPRODUCER.len(), 5966);
    assert!(expression_nesting_depth(REPRODUCER) > MAX_EXPRESSION_NESTING_DEPTH);
    assert!(!expression_is_safe_to_parse(REPRODUCER));
}

#[test]
fn oxc_reads_a_prefix_bang_after_a_closing_angle_as_a_regex_operand() {
    let allocator = Allocator::default();
    let source_type = SourceType::default()
        .with_module(true)
        .with_typescript(true);
    let parsed = Parser::new(&allocator, "a<b>!/`/", source_type).parse_expression();
    let Ok(Expression::BinaryExpression(outer)) = parsed else {
        panic!("relational chain: {parsed:?}");
    };
    assert_eq!(outer.operator, BinaryOperator::GreaterThan);
    let Expression::UnaryExpression(bang) = &outer.right else {
        panic!("prefix bang: {:?}", outer.right);
    };
    assert_eq!(bang.operator, UnaryOperator::LogicalNot);
    assert!(matches!(bang.argument, Expression::RegExpLiteral(_)));
}

#[test]
fn prefix_bang_after_a_closing_angle_keeps_the_following_brackets_live() {
    // OXC reads `a < b > !/`/` and parses every parenthesis after the regex;
    // reading the `/` as division used to open a template over the whole run.
    for prefix in [
        "a<b>!/`/",
        "a<b<c>>!!!/`/",
        "a<b> /* c */ !/`/",
        "x<y>\t!/`/",
    ] {
        let source = paren_run_behind(prefix);
        assert!(
            expression_nesting_depth(&source) > MAX_EXPRESSION_NESTING_DEPTH,
            "{source:?}"
        );
        assert!(!expression_is_safe_to_parse(&source), "{source:?}");
    }
}

#[test]
fn type_argument_division_and_inequality_keep_their_existing_reading() {
    for source in [
        "a<b>!=c",
        "a<b>!==c",
        "Props<{ a: 1 }>/2",
        "!(a<b>!c)",
        "a<b>!c && d",
    ] {
        assert!(expression_is_safe_to_parse(source), "{source:?}");
    }
    // After a line break OXC may keep the type arguments, so the `/` stays
    // division and the backtick opens a real template over the run.
    let split = paren_run_behind("a<b>\n!/`/");
    assert!(expression_nesting_depth(&split) <= MAX_EXPRESSION_NESTING_DEPTH);
}
