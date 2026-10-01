mod fixture;
use super::alias;
use crate::l4::{AdmissionFailure, LegacyReason};
use vize_l0::Allocator;

#[test]
fn native_formal_cannot_enter_ssr_string_or_vnode_parameter_text() {
    let a = Allocator::default();
    let actual = fixture::native(&a);
    let l3 = vize_l2_to_l3::lower(&a, &actual.root);
    let value = l3
        .program
        .operands
        .iter()
        .find(|op| op.role == vize_l3::operand::OperandRole::ForValue)
        .unwrap();
    assert_eq!(value.value.qualifier, "native-binding-unsupported");
    assert!(matches!(
        alias(&actual.binding),
        Err(AdmissionFailure::Unsupported(
            LegacyReason::ExpressionOrEncoding
        ))
    ));
    assert!(matches!(
        super::super::vnode_control::alias_source(&actual.binding),
        Err(AdmissionFailure::Unsupported(
            LegacyReason::ExpressionOrEncoding
        ))
    ));
    let options = crate::options::SsrCompilerOptions::default();
    assert!(crate::l4::compile_l2_to_ssr(&a, fixture::SOURCE, &actual.root, &options).is_none());
    assert!(matches!(
        crate::l4::l2_input::select_l2_lane(&a, fixture::SOURCE, &actual.root, &options),
        crate::l4::SsrL4Selection::Legacy(LegacyReason::ExpressionOrEncoding)
    ));
    let old = fixture::compatibility(&a);
    assert_eq!(alias(&old.binding).unwrap(), "x");
    assert_eq!(
        super::super::vnode_control::alias_source(&old.binding).unwrap(),
        "x"
    );
}
