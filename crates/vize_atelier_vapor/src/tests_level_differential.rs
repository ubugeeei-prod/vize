//! Davinci P1-7 differential lane — plain-suite coverage witness (vapor).
//!
//! Under `cfg(test)` the retained resolve path dual-runs every gated read
//! against the legacy `resolve_with_oxc` expression branch and panics on
//! divergence (`generate/expression_retained.rs`), so the vapor unit suite
//! is a differential run. This test plants retained-resolvable expressions
//! and asserts the comparator fired — a `cfg` regression that silently
//! disarmed the lane fails here. Counter deltas use `>=` (process-global
//! counters, concurrent tests); the exact-pinned lane lives in
//! `vize_atelier_sfc/tests/davinci_differential.rs`.

use crate::{VaporCompilerOptions, compile_vapor};
use vize_atelier_core::retained::differential;
use vize_carton::Allocator;

#[test]
fn retained_resolves_are_dual_run_across_the_unit_suite() {
    let before = differential::stats();

    let allocator = Allocator::new();
    let result = compile_vapor(
        &allocator,
        r#"<div :style="{ zIndex: items.length + 1 }" @click="handle($event, 'x')">
  {{ items.filter(item => item.id > 0).length / total }}
</div>"#,
        VaporCompilerOptions::default(),
    );
    assert!(!result.code.is_empty());

    let after = differential::stats();
    assert!(
        after.vapor_resolve_comparisons > before.vapor_resolve_comparisons,
        "vapor resolve dual-runs did not fire for the planted battery: {before:?} -> {after:?}"
    );
}
