use super::{JsCoordinateError, JsCoordinates, JsSegment};
use crate::expr::JsExpr;
use oxc_span::{GetSpan, SourceType};
use vize_l0::{Allocator, Span};

#[test]
fn retained_wrapper_corrects_ranges_without_copying_the_ast() {
    let a = Allocator::default();
    let expression = a.alloc(
        oxc_parser::Parser::new(a.as_oxc(), "(\n作者 + value\n)", SourceType::ts())
            .parse_expression()
            .unwrap(),
    );
    let oxc_ast::ast::Expression::ParenthesizedExpression(wrapper) = expression else {
        panic!("wrapper");
    };
    let ast = &wrapper.expression;
    let source = "作者 + value";
    let file = "--作者 + value--";
    let span = Span::new(2, 16);
    let coordinates = JsCoordinates::checked(file, source, span, 2, &[]).unwrap();
    let js = JsExpr::from_retained_in(&a, ast, source, span, coordinates).unwrap();
    assert!(core::ptr::eq(js.ast, ast));
    assert!(js.matches_authored_source(file));
    assert!(!js.matches_authored_source("--作者 + other--"));
    assert_eq!(js.ast_span_to_source(ast.span()), Some(Span::new(0, 14)));
    assert_eq!(js.authored_span(Span::new(0, 6)), Some(Span::new(2, 8)));
    assert_eq!(js.ast_span_to_source(oxc_span::Span::new(0, 1)), None);
    assert_eq!(js.ast_span_to_source(oxc_span::Span::new(2, 17)), None);
    assert_eq!(js.authored_span(Span::new(1, 2)), None);
}

#[test]
fn equal_byte_length_entities_still_refuse_interior_edits() {
    let file = "x&acE;y";
    let source = "x∾\u{333}y";
    let segments = [
        JsSegment {
            decoded: Span::new(0, 1),
            authored: Span::new(0, 1),
            entity: false,
        },
        JsSegment {
            decoded: Span::new(1, 6),
            authored: Span::new(1, 6),
            entity: true,
        },
        JsSegment {
            decoded: Span::new(6, 7),
            authored: Span::new(6, 7),
            entity: false,
        },
    ];
    let coordinates = JsCoordinates::checked(file, source, Span::new(0, 7), 2, &segments).unwrap();
    assert_eq!(
        coordinates.authored_span(Span::new(1, 6)),
        Some(Span::new(1, 6))
    );
    assert_eq!(coordinates.authored_span(Span::new(1, 4)), None);
    assert_eq!(coordinates.authored_span(Span::new(4, 4)), None);
    assert_eq!(
        coordinates.authored_span(Span::new(1, 1)),
        Some(Span::new(1, 1))
    );
    assert_eq!(
        coordinates.authored_span(Span::new(6, 6)),
        Some(Span::new(6, 6))
    );
}

#[test]
fn forged_or_incomplete_correspondences_do_not_enter_l2() {
    assert_eq!(
        JsCoordinates::checked("&amp;", "&", Span::new(0, 5), 0, &[]).unwrap_err(),
        JsCoordinateError::InvalidIdentity
    );
    let incomplete = [JsSegment {
        decoded: Span::new(0, 1),
        authored: Span::new(0, 1),
        entity: false,
    }];
    assert_eq!(
        JsCoordinates::checked("ab", "ab", Span::new(0, 2), 0, &incomplete).unwrap_err(),
        JsCoordinateError::InvalidCoverage
    );
    let non_entity = [JsSegment {
        decoded: Span::new(0, 1),
        authored: Span::new(0, 1),
        entity: true,
    }];
    assert_eq!(
        JsCoordinates::checked("a", "b", Span::new(0, 1), 0, &non_entity).unwrap_err(),
        JsCoordinateError::InvalidEntity
    );
    assert_eq!(
        JsCoordinates::checked("é", "é", Span::new(1, 2), 0, &[]).unwrap_err(),
        JsCoordinateError::InvalidSpan
    );
}

#[test]
fn identity_load_path_keeps_original_text_and_spans() {
    let a = Allocator::default();
    let js = JsExpr::parse_in(&a, "value", Span::new(10, 15)).unwrap();
    assert!(js.coordinates.is_none());
    assert_eq!(js.ast_span_to_source(js.ast.span()), Some(Span::new(0, 5)));
    assert_eq!(js.authored_span(Span::new(0, 5)), Some(Span::new(10, 15)));
}

#[test]
fn partial_expression_ast_cannot_claim_a_complete_source() {
    let a = Allocator::default();
    let ast = a.alloc(
        oxc_parser::Parser::new(a.as_oxc(), "a", SourceType::ts())
            .parse_expression()
            .unwrap(),
    );
    let coordinates = JsCoordinates::checked("a + b", "a + b", Span::new(0, 5), 0, &[]).unwrap();
    assert_eq!(
        JsExpr::from_retained_in(&a, ast, "a + b", Span::new(0, 5), coordinates).unwrap_err(),
        JsCoordinateError::OutsideExpression
    );
    let source = "a /* retained trivia */";
    let span = Span::new(0, source.len() as u32);
    let coordinates = JsCoordinates::checked(source, source, span, 0, &[]).unwrap();
    assert!(JsExpr::from_retained_in(&a, ast, source, span, coordinates).is_ok());
}
