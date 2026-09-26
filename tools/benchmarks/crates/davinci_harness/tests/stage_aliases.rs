//! Davinci stage aliases resolve as real crate names for implementation code.

use vize_l0::Allocator;
use vize_l2::{folio::DisegnoFolio, verify::Rigor};

#[test]
fn l0_l1_l2_and_l1_to_l2_aliases_compile_as_crate_names() {
    let allocator = Allocator::new();
    let (tree, errors) = vize_l1::parse(&allocator, "<div>{{ msg }}</div>");
    let lowered = vize_l1_to_l2::lower(&allocator, &tree, &errors);
    let folio = DisegnoFolio::of(&lowered.root.ops);
    let diagnostics = vize_l2::verify::verify(&folio, Rigor::Raw);

    assert!(errors.is_empty());
    assert!(diagnostics.is_empty());
}
