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
    assert_eq!(facts, expected(&file, expression, "yes ? '😀' : 'no'"));
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
