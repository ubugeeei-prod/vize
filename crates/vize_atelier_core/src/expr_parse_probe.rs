//! Temporary expression re-parse counter (Davinci P0-3).
//!
//! Counts every oxc parse of template-expression text on the compile path -
//! one increment per parse-scoped `oxc_allocator::Allocator` the transform,
//! codegen, and Vapor generate stages create today. The P0-3 benches read the
//! counter to record the per-fixture re-parse baseline
//! (`davinci-road/plan/expr-reparse-baseline.md`); phase 1 replaces the
//! parse-copy-reparse round trips with retained ASTs, TS-26 then asserts
//! `davinci.expr.parses == distinct expressions`, and this module is deleted
//! with the last call site.
//!
//! One relaxed atomic increment per counted parse; each counted parse builds
//! its own arena and runs a full oxc parse, so the probe is noise next to the
//! work it counts. The counter is process-global and monotone - readers diff
//! two loads around the region they care about.

use core::sync::atomic::{AtomicU64, Ordering};

static EXPR_PARSES: AtomicU64 = AtomicU64::new(0);

/// Count one expression parse. Placed next to every parse-scoped arena
/// creation on the compile path.
#[inline]
pub fn note_expr_parse() {
    EXPR_PARSES.fetch_add(1, Ordering::Relaxed);
}

/// Total counted expression parses since process start (monotone).
#[inline]
pub fn expr_parse_count() -> u64 {
    EXPR_PARSES.load(Ordering::Relaxed)
}
