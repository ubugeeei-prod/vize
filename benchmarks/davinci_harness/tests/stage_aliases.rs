//! Davinci stage aliases resolve as real crate names for implementation code.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

use vize_carton::Allocator;
use vize_s2::{folio::DisegnoFolio, verify::Rigor};

#[test]
fn s1_s2_and_s1_to_s2_aliases_compile_as_crate_names() {
    let allocator = Allocator::new();
    let (tree, errors) = vize_s1::parse(&allocator, "<div>{{ msg }}</div>");
    let lowered = vize_s1_to_s2::lower(&allocator, &tree, &errors);
    let folio = DisegnoFolio::of(&lowered.root.ops);
    let diagnostics = vize_s2::verify::verify(&folio, Rigor::Raw);

    assert!(errors.is_empty());
    assert!(diagnostics.is_empty());
}
