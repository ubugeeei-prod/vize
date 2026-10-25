use super::score;
use crate::analysis::template_complexity::{ComplexityFacts, Contribution, DecisionKind};
use crate::expr::js::{JsCoordinates, JsSegment};
use crate::expr::{ExprRef, JsExpr};
use oxc_ast::ast::Expression;
use oxc_span::SourceType;
use vize_l0::{Allocator, Span, id::NodeId};

fn span(source: &str, text: &str) -> Span {
    let start = source.find(text).unwrap() as u32;
    Span::new(start, start + text.len() as u32)
}

fn expected(source: &str, whole: &str, inner: &str) -> ComplexityFacts {
    ComplexityFacts {
        cyclomatic: 3,
        cognitive: 2,
        unknown: 0,
        max_nesting: 0,
        contributions: alloc::vec![
            Contribution {
                span: span(source, whole),
                kind: DecisionKind::Logical,
                op: NodeId::from_index(3),
                nesting: 0,
                cyclomatic: 1,
                cognitive: 1,
            },
            Contribution {
                span: span(source, inner),
                kind: DecisionKind::Conditional,
                op: NodeId::from_index(3),
                nesting: 0,
                cyclomatic: 1,
                cognitive: 1,
            },
        ],
    }
}

#[test]
fn checked_wrapper_coordinates_preserve_whole_logical_and_nested_conditional_ranges() {
    let a = Allocator::default();
    let expression = "ready && (yes ? '😀' : 'no')";
    for newline in ["\n", "\r\n"] {
        let file = alloc::format!("作者😀{newline}<p>{{{{ {expression} }}}}</p>");
        let wrapped = alloc::format!("({newline}{expression}{newline})");
        let ast = oxc_parser::Parser::new(a.as_oxc(), &wrapped, SourceType::ts())
            .parse_expression()
            .unwrap();
        let Expression::ParenthesizedExpression(wrapper) = a.alloc(ast) else {
            panic!("original parser wrapper");
        };
        let authored = span(&file, expression);
        let coordinates =
            JsCoordinates::checked(&file, expression, authored, (1 + newline.len()) as u32, &[])
                .unwrap();
        let js =
            JsExpr::from_retained_in(&a, &wrapper.expression, expression, authored, coordinates)
                .unwrap();
        let mut facts = ComplexityFacts::empty();
        score(ExprRef::Js(js), NodeId::from_index(3), 0, &mut facts);
        facts
            .contributions
            .sort_unstable_by_key(Contribution::sort_key);
        assert_eq!(facts, expected(&file, expression, "yes ? '😀' : 'no'"));
        assert!(core::ptr::eq(js.ast, &wrapper.expression));
    }
}

#[test]
fn load_path_identity_coordinates_retain_the_complete_existing_fact_shape() {
    let a = Allocator::default();
    let expression = "ready && (yes ? '😀' : 'no')";
    let file = alloc::format!("作者😀\r\n<p>{{{{ {expression} }}}}</p>");
    let js = JsExpr::parse_in(&a, expression, span(&file, expression)).unwrap();
    let mut facts = ComplexityFacts::empty();
    score(ExprRef::Js(js), NodeId::from_index(3), 0, &mut facts);
    facts
        .contributions
        .sort_unstable_by_key(Contribution::sort_key);
    assert_eq!(facts, expected(&file, expression, "yes ? '😀' : 'no'"));
}

#[test]
fn identity_projection_keeps_utf8_bounds_and_authored_overflow_refusals() {
    let a = Allocator::default();
    let expression = "ready && '😀'";
    let inside_unicode = expression.find('😀').unwrap() as u32 + 1;
    for (ast_span, authored) in [
        (
            oxc_span::Span::new(inside_unicode, expression.len() as u32),
            Span::new(10, 10 + expression.len() as u32),
        ),
        (
            oxc_span::Span::new(0, expression.len() as u32 + 1),
            Span::new(10, 10 + expression.len() as u32),
        ),
        (
            oxc_span::Span::new(0, expression.len() as u32),
            Span::new(u32::MAX - 1, u32::MAX),
        ),
        (
            oxc_span::Span::new(0, expression.len() as u32),
            Span::new(10, 11),
        ),
    ] {
        let mut ast = JsExpr::parse_ast_in(&a, expression).unwrap();
        let Expression::LogicalExpression(logical) = &mut ast else {
            panic!("logical test expression");
        };
        logical.span = ast_span;
        let js = JsExpr {
            ast: a.alloc(ast),
            source: expression,
            span: authored,
            coordinates: None,
        };
        let mut facts = ComplexityFacts::empty();
        score(ExprRef::Js(&js), NodeId::from_index(3), 2, &mut facts);
        assert_eq!(
            facts,
            ComplexityFacts {
                cyclomatic: 1,
                cognitive: 0,
                unknown: 1,
                max_nesting: 0,
                contributions: alloc::vec![Contribution {
                    span: authored,
                    kind: DecisionKind::Unknown,
                    op: NodeId::from_index(3),
                    nesting: 2,
                    cyclomatic: 0,
                    cognitive: 0,
                }],
            }
        );
    }
}

#[test]
fn terminal_leaves_preserve_facts_and_compounds_still_visit_nested_decisions() {
    let a = Allocator::default();
    for expression in [
        "name",
        "true",
        "null",
        "42",
        "42n",
        "/a/",
        "'😀'",
        "this",
        "obj.value.deep",
        "((obj.value).deep)",
    ] {
        let js = JsExpr::parse_in(&a, expression, Span::new(0, expression.len() as u32)).unwrap();
        let mut facts = expected("a && (b ? c : d)", "a && (b ? c : d)", "b ? c : d");
        let previous = facts.clone();
        score(ExprRef::Js(js), NodeId::from_index(4), 3, &mut facts);
        assert_eq!(facts, previous, "terminal {expression}");
    }
    for expression in [
        "`value ${a && (b ? c : d)}`",
        "obj[a && (b ? c : d)]",
        "[a && (b ? c : d)]",
        "({ value: a && (b ? c : d) })",
        "fn(a && (b ? c : d))",
        "(() => a && (b ? c : d))",
        "(a && (b ? c : d)).value.deep",
    ] {
        let js = JsExpr::parse_in(&a, expression, Span::new(0, expression.len() as u32)).unwrap();
        let mut facts = ComplexityFacts::empty();
        score(ExprRef::Js(js), NodeId::from_index(3), 0, &mut facts);
        facts
            .contributions
            .sort_unstable_by_key(Contribution::sort_key);
        assert_eq!(
            facts,
            expected(expression, "a && (b ? c : d)", "b ? c : d"),
            "compound {expression}",
        );
    }
}

#[test]
fn static_members_keep_logical_trees_separate_across_the_object_boundary() {
    let a = Allocator::default();
    let expression = "((a && b).value) || c";
    let js = JsExpr::parse_in(&a, expression, Span::new(0, expression.len() as u32)).unwrap();
    let mut facts = ComplexityFacts::empty();
    score(ExprRef::Js(js), NodeId::from_index(3), 0, &mut facts);
    facts
        .contributions
        .sort_unstable_by_key(Contribution::sort_key);
    assert_eq!(
        facts,
        ComplexityFacts {
            cyclomatic: 3,
            cognitive: 2,
            unknown: 0,
            max_nesting: 0,
            contributions: alloc::vec![
                Contribution {
                    span: span(expression, expression),
                    kind: DecisionKind::Logical,
                    op: NodeId::from_index(3),
                    nesting: 0,
                    cyclomatic: 1,
                    cognitive: 1
                },
                Contribution {
                    span: span(expression, "a && b"),
                    kind: DecisionKind::Logical,
                    op: NodeId::from_index(3),
                    nesting: 0,
                    cyclomatic: 1,
                    cognitive: 1
                },
            ],
        }
    );
}

#[test]
fn entity_interiors_refuse_the_whole_expression_instead_of_publishing_partial_facts() {
    let a = Allocator::default();
    let expression = "ready && (yes ? '😀' : 'no')";
    // Coordinate custody permits an indivisible decoded segment. This law
    // checks the scorer's refusal; it is not an HTML entity decoder fixture.
    let file = "作者😀&opaque;";
    let authored = span(file, "&opaque;");
    let segments = [JsSegment {
        decoded: Span::new(0, expression.len() as u32),
        authored,
        entity: true,
    }];
    let coordinates = JsCoordinates::checked(file, expression, authored, 0, &segments).unwrap();
    let ast = a.alloc(
        oxc_parser::Parser::new(a.as_oxc(), expression, SourceType::ts())
            .parse_expression()
            .unwrap(),
    );
    let js = JsExpr::from_retained_in(&a, ast, expression, authored, coordinates).unwrap();
    let prior = Contribution {
        span: Span::new(0, 0),
        kind: DecisionKind::Conditional,
        op: NodeId::from_index(1),
        nesting: 0,
        cyclomatic: 1,
        cognitive: 1,
    };
    let mut facts = ComplexityFacts::empty();
    facts.push(prior);
    score(ExprRef::Js(js), NodeId::from_index(3), 2, &mut facts);
    assert_eq!(
        facts,
        ComplexityFacts {
            cyclomatic: 2,
            cognitive: 1,
            unknown: 1,
            max_nesting: 0,
            contributions: alloc::vec![
                prior,
                Contribution {
                    span: authored,
                    kind: DecisionKind::Unknown,
                    op: NodeId::from_index(3),
                    nesting: 2,
                    cyclomatic: 0,
                    cognitive: 0,
                },
            ],
        }
    );
}
