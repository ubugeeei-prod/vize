use alloc::vec::Vec;
use oxc_ast::ast::Expression;
use oxc_diagnostics::{LabeledSpan, OxcDiagnostic};
use oxc_span::GetSpan;
use vize_l0::{Allocator, Span};

use super::{
    Coordinates, DiagnosticView, Embed, EmbedHole, EmbedSource, Grammar, Lang, Shape, SourceError,
    parse_once, parser_length,
};
use crate::embed::prepare_attribute_value;

fn raw_embed(text: &str, shape: Shape, lang: Lang) -> Embed<'_> {
    Embed {
        grammar: Grammar { shape, lang },
        source: EmbedSource::authored(text, Span::new(0, text.len() as u32)).unwrap(),
    }
}

#[test]
fn js_and_ts_programs_retain_real_ast_comments_and_original_source() {
    let allocator = Allocator::default();
    for (text, lang) in [
        ("/* keep */ const value = 1; // tail", Lang::Js),
        ("/* keep */ const value: number = 1; // tail", Lang::Ts),
    ] {
        let tree = parse_once(&allocator, raw_embed(text, Shape::Program, lang));
        assert_eq!(tree.hole(), None);
        let program = tree.program().unwrap();
        assert_eq!(program.source_text, text);
        assert_eq!(program.body.len(), 1);
        assert!(tree.expression().is_none());
        let comments: Vec<_> = tree
            .comments()
            .map(|comment| comment.text().unwrap())
            .collect();
        assert_eq!(comments, ["/* keep */", "// tail"]);
        assert_eq!(tree.diagnostics().count(), 0);
        assert_eq!(
            tree.decoded_span(program.span),
            Ok(Span::new(0, text.len() as u32))
        );
    }
}

#[test]
fn expressions_hide_the_wrapper_and_preserve_authored_parentheses_and_comments() {
    let allocator = Allocator::default();
    for lang in [Lang::Js, Lang::Ts] {
        let tree = parse_once(
            &allocator,
            raw_embed("/* lead */ (count) // tail", Shape::Expr, lang),
        );
        assert_eq!(tree.hole(), None);
        assert!(tree.program().is_none());
        let expression = tree.expression().unwrap();
        assert!(matches!(expression, Expression::ParenthesizedExpression(_)));
        assert_eq!(tree.decoded_span(expression.span()), Ok(Span::new(11, 18)));
        let comments: Vec<_> = tree.comments().collect();
        assert_eq!(comments.len(), 2);
        assert_eq!(comments[0].text().unwrap(), "/* lead */");
        assert_eq!(comments[0].decoded_span(), Ok(Span::new(0, 10)));
        assert_eq!(comments[1].text().unwrap(), "// tail");
        assert_eq!(comments[1].decoded_span(), Ok(Span::new(19, 26)));
        assert_eq!(
            tree.decoded_span(oxc_span::Span::new(0, 1)),
            Err(SourceError::InvalidDecodedSpan)
        );
    }
}

#[test]
fn decoded_attribute_is_the_actual_parser_input_and_ast_edits_stay_exact() {
    let allocator = Allocator::default();
    let source = prepare_attribute_value(&allocator, "xx&fjlig;yy", Span::new(2, 9)).unwrap();
    let tree = parse_once(
        &allocator,
        Embed {
            grammar: Grammar {
                shape: Shape::Expr,
                lang: Lang::Js,
            },
            source,
        },
    );
    let Expression::Identifier(identifier) = tree.expression().unwrap() else {
        panic!("expected decoded identifier")
    };
    assert_eq!(identifier.name.as_str(), "fj");
    assert_eq!(tree.decoded_span(identifier.span), Ok(Span::new(0, 2)));
    assert_eq!(tree.authored_span(identifier.span), Ok(Span::new(2, 9)));
    assert_eq!(
        tree.authored_span(oxc_span::Span::new(2, 3)),
        Err(SourceError::PartialEntityBoundary)
    );
    assert_eq!(tree.source().text(), "fj");
}

#[test]
fn syntax_failures_keep_comments_and_corrected_diagnostics_but_hide_recovery_ast() {
    let allocator = Allocator::default();
    let text = "count + /* kept */";
    let tree = parse_once(&allocator, raw_embed(text, Shape::Expr, Lang::Js));
    assert_eq!(tree.hole(), Some(EmbedHole::Syntax));
    assert!(tree.expression().is_none() && tree.program().is_none());
    assert_eq!(tree.source().text(), text);
    assert_eq!(
        tree.comments().next().unwrap().text().unwrap(),
        "/* kept */"
    );
    let diagnostics: Vec<_> = tree.diagnostics().collect();
    assert!(!diagnostics.is_empty());
    let labels: Vec<_> = diagnostics
        .iter()
        .flat_map(|diagnostic| diagnostic.labels())
        .collect();
    assert!(!labels.is_empty());
    for diagnostic in diagnostics {
        assert!(!diagnostic.message().is_empty());
    }
    for label in &labels {
        let decoded = label.decoded_span().unwrap();
        let authored = label.authored_span().unwrap();
        assert_eq!(decoded, authored);
        assert!(
            text.get(decoded.start as usize..decoded.end as usize)
                .is_some()
        );
    }
    assert!(
        labels.iter().any(
            |label| label.decoded_span() == Ok(Span::new(text.len() as u32, text.len() as u32))
        )
    );
}

#[test]
fn diagnostic_entity_points_cover_authored_reference_without_weakening_edit_projection() {
    let allocator = Allocator::default();
    let source = prepare_attribute_value(&allocator, "xx&acE;yy", Span::new(2, 7)).unwrap();
    let coordinates = Coordinates { source, prefix: 2 };
    let diagnostic = OxcDiagnostic::error("inner scalar").with_label(LabeledSpan::new(None, 5, 0));
    let view = DiagnosticView {
        diagnostic: &diagnostic,
        coordinates,
    };
    let label = view.labels().next().unwrap();
    assert_eq!(label.decoded_span(), Ok(Span::new(3, 3)));
    assert_eq!(label.authored_span(), Ok(Span::new(2, 7)));
    assert_eq!(
        source.authored_span(Span::new(3, 3)),
        Err(SourceError::PartialEntityBoundary)
    );
}

#[test]
fn exact_coordinates_reject_utf8_splits_and_diagnostic_coordinates_cover_scalars() {
    let source = EmbedSource::authored("xαβy", Span::new(1, 5)).unwrap();
    let coordinates = Coordinates { source, prefix: 2 };
    assert_eq!(
        coordinates.decoded_span(oxc_span::Span::new(3, 3)),
        Err(SourceError::InvalidDecodedSpan)
    );
    assert_eq!(coordinates.diagnostic_span(3, 3), Ok(Span::new(0, 2)));
    assert_eq!(coordinates.diagnostic_span(0, 1), Ok(Span::new(0, 0)));
    assert_eq!(coordinates.diagnostic_span(6, 7), Ok(Span::new(4, 4)));
    assert_eq!(
        coordinates.diagnostic_span(4, 3),
        Err(SourceError::InvalidDecodedSpan)
    );
}

#[test]
fn unsupported_shapes_and_bounded_admission_are_typed_local_holes() {
    let allocator = Allocator::default();
    for shape in [Shape::ForHead, Shape::FilterChain] {
        let tree = parse_once(&allocator, raw_embed("count", shape, Lang::Js));
        assert_eq!(tree.hole(), Some(EmbedHole::UnsupportedShape));
        assert_eq!(tree.grammar().shape, shape);
        assert_eq!(tree.source().text(), "count");
        assert!(tree.expression().is_none() && tree.program().is_none());
    }
    let text = vize_l0::cstr!("{}x{}", "(".repeat(32), ")".repeat(32));
    let tree = parse_once(&allocator, raw_embed(text.as_str(), Shape::Expr, Lang::Js));
    assert_eq!(tree.hole(), Some(EmbedHole::TokenBudget));
    let tree = parse_once(&allocator, raw_embed("(count", Shape::Expr, Lang::Js));
    assert_eq!(tree.hole(), Some(EmbedHole::SafetyAdmission));
}

#[test]
fn ts_syntax_requires_ts_admission_and_statement_escapes_are_not_expressions() {
    let allocator = Allocator::default();
    for (lang, expected) in [(Lang::Js, Some(EmbedHole::Syntax)), (Lang::Ts, None)] {
        let tree = parse_once(&allocator, raw_embed("count as number", Shape::Expr, lang));
        assert_eq!(tree.hole(), expected);
    }
    for text in ["count; other", "count) + (other"] {
        let tree = parse_once(&allocator, raw_embed(text, Shape::Expr, Lang::Js));
        assert!(tree.hole().is_some());
        assert!(tree.expression().is_none());
    }
}

#[test]
fn parser_size_admission_checks_wrapper_and_machine_limits_before_allocation() {
    assert_eq!(parser_length(1, 4), Some(5));
    assert_eq!(parser_length(usize::MAX, 4), None);
    assert_eq!(parser_length(u32::MAX as usize, 4), None);
    assert_eq!(parser_length(isize::MAX as usize, 4), None);
}
