//! Davinci P1-7 differential lane — plain-suite coverage witness.
//!
//! Under `cfg(test)` the migrated atelier sites dual-run every retained-AST
//! read against the legacy re-parse and panic on divergence (see
//! [`crate::retained`]), so the whole atelier_core unit suite is a
//! differential run. This test plants a battery through both the transform
//! lane (prefixing on — the SSR shape) and the codegen lane (prefixing
//! deferred to codegen — the cached-handler shape) and asserts the
//! comparators actually fired — a `cfg` regression that silently disarmed
//! the lane fails here. Counter deltas use `>=`: the counters are
//! process-global and other unit tests run concurrently. The exact-pinned
//! lane lives in `vize_atelier_sfc/tests/davinci_differential.rs` (own
//! process, deterministic counts).

use crate::codegen::generate_with_sections;
use crate::lane::transform;
use crate::options::{CodegenOptions, TransformOptions};
use crate::parser::Parser;
use crate::retained::differential;
use vize_s0::Allocator;

const BATTERY: &str = r#"<div :style="{ zIndex: items.length + 1 }" :data-x="alpha + beta">
  {{ items.filter(item => item.id > 0).length / total }}
  <button @click="handle($event)" @keyup="count++">go</button>
</div>"#;

#[test]
fn retained_reads_are_dual_run_across_the_unit_suite() {
    let before = differential::stats();

    // Transform-lane pass (prefixing on): interpolations and bound values go
    // through `rewrite_expression`'s retained walk.
    let allocator = Allocator::new();
    let (mut root, errors) = Parser::new(&allocator, BATTERY).parse();
    assert_eq!(errors.len(), 0, "battery must parse cleanly: {errors:?}");
    transform(
        &allocator,
        &mut root,
        TransformOptions {
            prefix_identifiers: true,
            ..TransformOptions::default()
        },
        None,
    );

    let mid = differential::stats();
    assert!(
        mid.transform_rewrite_comparisons > before.transform_rewrite_comparisons,
        "transform-lane rewrite dual-runs did not fire for the planted battery: {before:?} -> {mid:?}"
    );

    // Codegen-lane pass (transform without prefixing, codegen with it): the
    // event-handler shape checks and the codegen prefixer walk their
    // retained ASTs; patch-flag classification walks the `:style` object.
    let allocator = Allocator::new();
    let (mut root, errors) = Parser::new(&allocator, BATTERY).parse();
    assert_eq!(errors.len(), 0, "battery must parse cleanly: {errors:?}");
    transform(&allocator, &mut root, TransformOptions::default(), None);
    let _generated = generate_with_sections(
        &root,
        CodegenOptions {
            prefix_identifiers: true,
            ..CodegenOptions::default()
        },
    );

    let after = differential::stats();
    assert!(
        after.shape_comparisons > before.shape_comparisons,
        "shape-check dual-runs did not fire for the planted battery: {before:?} -> {after:?}"
    );
    assert!(
        after.codegen_rewrite_comparisons > mid.codegen_rewrite_comparisons,
        "codegen-lane rewrite dual-runs did not fire for the planted battery: {mid:?} -> {after:?}"
    );
}
