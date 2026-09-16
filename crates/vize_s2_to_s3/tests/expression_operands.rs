use vize_s0::{Allocator, Box, Span, Vec};
use vize_s2::expr::{ExprRef, ForeignExpr, OpaqueExpr, OpaqueReason, VueFilterExpr};
use vize_s2::op::{InterpolationOp, Op, Region};
use vize_s2_to_s3::lower;
use vize_s3::operand::{OperandRole, ValueKind};
use vize_s3::verify::verify;

#[test]
fn lowering_copies_expression_classification_without_reinterpreting_source() {
    let s3_arena = Allocator::default();
    let lowered = {
        let arena = Allocator::default();
        let source = arena.alloc_str("value");
        let span = Span::new(0, 5);
        let dialect = arena.alloc_str("moonbit");
        let foreign = arena.alloc(ForeignExpr {
            dialect,
            source,
            span,
            facts: Vec::new_in(&&arena),
        });
        let opaque = arena.alloc(OpaqueExpr {
            reason: OpaqueReason::Compound,
            source,
            span,
        });
        let filter_source = arena.alloc_str("value | capitalize");
        let filter_span = Span::new(6, 24);
        let filter = VueFilterExpr::parse_in(&arena, filter_source, filter_span).unwrap();
        let expressions = [
            ExprRef::parse_js_in(&arena, source, span),
            ExprRef::Opaque(opaque),
            ExprRef::Foreign(foreign),
            ExprRef::Filter(filter),
        ];
        let mut ops = Vec::new_in(&&arena);
        for expression in expressions {
            ops.push(Op::Interpolation(Box::new_in(
                InterpolationOp {
                    expression,
                    span: expression.span(),
                },
                &&arena,
            )));
        }
        lower(&s3_arena, &Region { ops })
    };
    assert_eq!(verify(&lowered.program), []);
    let operands = &lowered.program.operands;
    assert_eq!(operands.len(), 4);
    for (operand, (kind, qualifier, source, span)) in operands.iter().zip([
        (ValueKind::Js, "", "value", Span::new(0, 5)),
        (ValueKind::Opaque, "compound", "value", Span::new(0, 5)),
        (ValueKind::Foreign, "moonbit", "value", Span::new(0, 5)),
        (
            ValueKind::Filter,
            "",
            "value | capitalize",
            Span::new(6, 24),
        ),
    ]) {
        assert_eq!(operand.role, OperandRole::Text);
        assert_eq!(operand.value.kind, kind);
        assert_eq!(operand.value.qualifier, qualifier);
        assert_eq!(operand.value.text, source);
        assert_eq!(operand.value.span, span);
    }
}
