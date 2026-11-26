//! The lower File factory can hold overlapping proofs and locally refused
//! expressions that a whole native SFC would never publish.
use super::{ExpressionPositionIndex, Model, PositionQueryError, TemplateQueryError};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_l0::{Allocator, SourceRoot, Span};
use vize_l2::{
    artifact::ComponentFactory,
    expr::JsExpr,
    file::{Declaration, TemplatePolicy, TemplateScope},
    lang::js::{L2IrProducer, ProgramInput, ProgramScope},
};

#[derive(Clone, Copy)]
struct Visible;
impl TemplatePolicy for Visible {
    fn visible(self, _: &Declaration) -> bool {
        true
    }
}

fn producer<'a>(arena: &'a Allocator, source: &'a str) -> L2IrProducer<'a> {
    let (script, _) = source.split_once('\n').unwrap();
    let block = SourceRoot::new(source).unwrap().block(script, 0).unwrap();
    let input = Parser::new(arena, script, SourceType::mjs()).parse_observed();
    let mut producer = L2IrProducer::new(arena, source).unwrap();
    producer
        .program(
            ProgramInput::checked(input.admitted().unwrap(), block, 0).unwrap(),
            ProgramScope::Nested,
        )
        .unwrap();
    producer
}

#[test]
fn duplicate_sites_report_ambiguity_even_after_moving_the_index() {
    let arena = Allocator::default();
    let source = "const n=0;\nn";
    let mut file = producer(&arena, source);
    let span = Span::new(source.len() as u32 - 1, source.len() as u32);
    let expression = JsExpr::parse_in(&arena, span.slice(source), span).unwrap();
    {
        let mut factory = file
            .template_region(TemplateScope::LastUnit, Visible)
            .unwrap();
        for _ in 0..16 {
            factory.interpolation(expression, span).unwrap();
        }
    }
    let file = file.finish().unwrap();
    let ambiguous = Some(TemplateQueryError::Position(
        PositionQueryError::AmbiguousSite,
    ));
    assert_eq!(
        Model::new(&file)
            .expression_reference_at_offset(span.start)
            .err(),
        ambiguous
    );
    let index = ExpressionPositionIndex::from_file(file);
    assert_eq!(
        index.expression_reference_at_offset(span.start).err(),
        ambiguous
    );
    assert_eq!(
        index.expression_reference_at_offset(span.start).err(),
        ambiguous
    );
    let moved = Box::new(index);
    assert_eq!(
        moved.expression_reference_at_offset(span.start).err(),
        ambiguous
    );
    assert!(
        moved
            .expression_reference_at_offset(span.end)
            .unwrap()
            .is_none()
    );
    let restored = moved.into_file();
    let mut total = 0;
    Model::new(&restored)
        .for_each_expression_reference(|_| total += 1)
        .unwrap();
    assert_eq!(total, 16);
}

#[test]
fn a_refused_expression_invalidates_earlier_uses() {
    let arena = Allocator::default();
    let source = "const n=0;\nn missing";
    let mut file = producer(&arena, source);
    {
        let mut region = file
            .template_region(TemplateScope::LastUnit, Visible)
            .unwrap();
        for word in ["n", "missing"] {
            let index = source.rfind(word).unwrap() as u32;
            let span = Span::new(index, index + word.len() as u32);
            let expression = JsExpr::parse_in(&arena, span.slice(source), span).unwrap();
            assert_eq!(region.interpolation(expression, span).is_ok(), word == "n");
        }
    }
    let file = file.finish().unwrap();
    let model = Model::new(&file);
    let mut calls = 0;
    let error = TemplateQueryError::Position(PositionQueryError::IncompleteFile);
    #[cfg(feature = "template-operand-queries")]
    assert_eq!(
        model.for_each_resolved_template_operand(|_| calls += 1),
        Err(error)
    );
    assert_eq!(
        model.for_each_expression_reference(|_| calls += 1),
        Err(error)
    );
    assert_eq!(calls, 0);
    let index = ExpressionPositionIndex::new(&file);
    for _ in 0..3 {
        assert_eq!(
            index.expression_reference_at_offset(u32::MAX).err(),
            Some(error)
        );
    }
}

#[test]
fn indivisible_source_segments_refuse_partial_tokens_in_both_lookups() {
    use vize_l2::expr::js::{JsCoordinates, JsSegment};
    let arena = Allocator::default();
    let source = "const n=0;\n&multi;";
    let start = source.find('&').unwrap() as u32;
    let span = Span::new(start, source.len() as u32);
    // Use the checked lower coordinate API: one source segment expands across
    // two identifiers. Neither occurrence can publish a partial segment.
    let segments = [JsSegment {
        decoded: Span::new(0, 3),
        authored: span,
        entity: true,
    }];
    let text = "n+n";
    let coordinates = JsCoordinates::checked(source, text, span, 0, &segments).unwrap();
    let syntax = arena.alloc(JsExpr::parse_ast_in(&arena, text).unwrap());
    let expression = JsExpr::from_retained_in(&arena, syntax, text, span, coordinates).unwrap();
    let mut file = producer(&arena, source);
    file.template_region(TemplateScope::LastUnit, Visible)
        .unwrap()
        .interpolation(expression, span)
        .unwrap();
    let file = file.finish().unwrap();
    assert!(file.is_complete());
    let model = Model::new(&file);
    assert_eq!(
        model.for_each_expression_reference(|_| {}).err(),
        Some(TemplateQueryError::Projection)
    );
    let index = ExpressionPositionIndex::new(&file);
    for offset in [0, start, start + 1] {
        assert_eq!(
            model.expression_reference_at_offset(offset).err(),
            Some(TemplateQueryError::Projection)
        );
        assert_eq!(
            index.expression_reference_at_offset(offset).err(),
            Some(TemplateQueryError::Projection)
        );
    }
}
