mod fixture;
use super::{optional_ident, value_alias};
use crate::emit::{EmitError, UnsupportedReason};
use vize_l0::{Allocator, Span};

#[test]
fn native_formal_is_refused_at_every_dom_alias_entry_while_expression_alias_stays_exact() {
    let a = Allocator::default();
    let actual = fixture::native(&a);
    let expected =
        EmitError::unsupported_at(UnsupportedReason::ForAliasNotEmittable, Span::new(1, 2));
    assert_eq!(value_alias(&actual.binding), Err(expected));
    assert_eq!(optional_ident(&Some(actual.binding)), Err(expected));
    assert_eq!(actual.root.ops.len(), 1);
    // A rejection fixture asserts no native producer scope/fact coverage.
    let (tree, errors) = vize_l1::parse(&a, "");
    let mut lowered = crate::lower::lower(&a, &tree, &errors);
    lowered.source = fixture::SOURCE;
    lowered.root = actual.root;
    lowered.op_count = 1;
    assert_eq!(
        crate::emit::emit_dom(&lowered, &crate::pass::L2Facts::default()),
        Err(expected)
    );
    let old = fixture::compatibility(&a);
    assert_eq!(value_alias(&old.binding), Ok("x"));
    assert_eq!(optional_ident(&Some(old.binding)), Ok(Some("x")));
}

#[cfg(debug_assertions)]
#[test]
fn public_transform_reports_native_inspection_failure_without_running_the_after_pass_hook() {
    let a = Allocator::default();
    let actual = fixture::native(&a);
    let (tree, errors) = vize_l1::parse(&a, "");
    let mut lowered = crate::lower::lower(&a, &tree, &errors);
    lowered.source = fixture::SOURCE;
    lowered.root = actual.root;
    lowered.op_count = 1;
    let mut observer = vize_l0::pass::BudgetObserver::new();
    let mut after_pass = 0;
    crate::pass::run_transform_with_pass_hook(
        &mut lowered,
        &mut observer,
        crate::pass::TransformProfile::DEFAULT,
        |_, _| after_pass += 1,
    );
    assert_eq!((observer.passes, observer.failures, after_pass), (1, 1, 0));
    assert_eq!(
        lowered.diagnostics,
        [crate::exemptions::lowering(
            Span::new(1, 2),
            "native binding dump unsupported",
        )]
    );
}

#[cfg(debug_assertions)]
#[test]
fn public_dom_legacy_transform_reports_the_actual_native_binding_span() {
    let a = Allocator::default();
    let actual = fixture::native(&a);
    let (tree, errors) = vize_l1::parse(&a, "");
    let mut lowered = crate::lower::lower(&a, &tree, &errors);
    lowered.source = fixture::SOURCE;
    lowered.root = actual.root;
    lowered.op_count = 1;
    lowered.caps = crate::lower::LegacyCaps::for_version(vize_l0::config::VueVersion::V2);
    let mut observer = vize_l0::pass::BudgetObserver::new();
    crate::pass::run_dom_transform_with_profile(
        &mut lowered,
        &mut observer,
        crate::pass::TransformProfile::DEFAULT,
    );
    assert_eq!((observer.passes, observer.failures), (1, 1));
    assert_eq!(
        lowered.diagnostics,
        [crate::exemptions::lowering(
            Span::new(1, 2),
            "native binding dump unsupported",
        )]
    );
}
