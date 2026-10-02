mod native_binding_fixture;
use native_binding_fixture as fixture;
use vize_l0::{Allocator, Span};
use vize_l2_to_l3::lower;
use vize_l3::{
    operand::{OperandRole as Role, ValueKind},
    verify::verify,
};

#[test]
fn retained_native_binding_survives_as_explicit_unsupported_operand_after_l2_drops() {
    let l3 = Allocator::default();
    let lowered = {
        let l2 = Allocator::default();
        let actual = fixture::native(&l2).unwrap();
        assert!(actual.binding.as_expression().is_none());
        lower(&l3, &actual.root)
    };
    assert_eq!(verify(&lowered.program), []);
    let alias = lowered
        .program
        .operands
        .iter()
        .find(|operand| operand.role == Role::ForValue)
        .unwrap();
    assert_eq!(alias.value.kind, ValueKind::Opaque);
    assert_eq!(alias.value.qualifier, "native-binding-unsupported");
    assert_eq!(alias.value.text, "x");
    assert_eq!(alias.value.span, Span::new(1, 2));
}

#[test]
fn expression_aliases_keep_their_original_classification_order_and_spans() {
    let a = Allocator::default();
    let actual = fixture::compatibility(&a);
    assert!(actual.binding.as_expression().is_some());
    let lowered = lower(&a, &actual.root);
    assert_eq!(verify(&lowered.program), []);
    let values = lowered
        .program
        .operands
        .iter()
        .map(|op| {
            (
                op.role,
                op.value.kind,
                op.value.text,
                op.value.qualifier,
                op.value.span,
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        values,
        [
            (
                Role::ForSource,
                ValueKind::Js,
                "items",
                "",
                Span::new(5, 10)
            ),
            (Role::ForValue, ValueKind::Js, "x", "", Span::new(1, 2)),
            (Role::ForKey, ValueKind::Absent, "", "", Span::new(5, 10)),
            (Role::ForIndex, ValueKind::Absent, "", "", Span::new(5, 10)),
        ]
    );
}
