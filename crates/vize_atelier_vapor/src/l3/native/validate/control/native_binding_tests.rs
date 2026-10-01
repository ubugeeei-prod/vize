mod fixture;
use super::{alias, for_loop};
use crate::l3::{AdmissionFailure, LegacyReason, retained::Retained};
use vize_l0::{Allocator, Span};
use vize_l3::operand::{OperandRole as Role, ValueKind};

#[test]
fn actual_lowered_native_binding_cannot_enter_vapor_loop_execution() {
    let a = Allocator::default();
    let actual = fixture::native(&a);
    let lowered = vize_l2_to_l3::lower(&a, &actual.root);
    let value = lowered
        .program
        .operands
        .iter()
        .find(|op| op.role == Role::ForValue)
        .unwrap()
        .value;
    assert_eq!(value.kind, ValueKind::Opaque);
    assert_eq!(value.qualifier, "native-binding-unsupported");
    assert_eq!(value.span, actual.binding.span());
    let retained = Retained::new(&a);
    assert!(matches!(
        for_loop(
            &lowered.program.operands,
            &retained,
            None,
            Span::new(0, fixture::SOURCE.len() as u32)
        ),
        Err(AdmissionFailure::Unsupported(LegacyReason::ControlFlow))
    ));
    assert!(matches!(
        alias(value.kind, value.text),
        Err(AdmissionFailure::Unsupported(LegacyReason::ControlFlow))
    ));
    let old = fixture::compatibility(&a);
    let old_l3 = vize_l2_to_l3::lower(&a, &old.root);
    let value = old_l3
        .program
        .operands
        .iter()
        .find(|op| op.role == Role::ForValue)
        .unwrap()
        .value;
    assert_eq!(alias(value.kind, value.text).unwrap(), Some("x"));
}
