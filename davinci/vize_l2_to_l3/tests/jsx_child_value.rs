use vize_l0::{Allocator, Box, Span, Vec};
use vize_l2::{
    expr::ExprRef,
    op::{InterpolationOp, Op, Region},
};
use vize_l2_to_l3::{PartitionKind, lower};
use vize_l3::{op::OpKind, operand::OperandRole, verify::verify};

#[test]
fn opaque_child_is_dynamic_and_keeps_its_semantic_role_and_authored_expression() {
    let allocator = Allocator::new();
    let span = Span::new(0, 11);
    let root = Region {
        ops: Vec::from_iter_in(
            [Op::JsxChildValue(Box::new_in(
                InterpolationOp {
                    expression: ExprRef::parse_js_in(&allocator, "state.value", span),
                    span,
                },
                &allocator,
            ))],
            &allocator,
        ),
    };
    let mut result = lower(&allocator, &root);
    assert_eq!(verify(&result.program), []);
    assert_eq!(result.program.ops.len(), 1);
    assert_eq!(result.program.ops[0].kind, OpKind::SetChildValue);
    assert_eq!(result.partition.ops[0].kind, PartitionKind::Dynamic);
    assert_eq!(result.program.operands.len(), 1);
    assert_eq!(result.program.operands[0].role, OperandRole::ChildValue);
    assert_eq!(result.program.operands[0].value.text, "state.value");
    assert_eq!(result.program.operands[0].value.span, span);
    result.program.operands[0].role = OperandRole::Text;
    assert!(
        !verify(&result.program).is_empty(),
        "a child value must never be retagged as display text"
    );
}
