//! Dual-run instrumentation for the Davinci differential lane (split from
//! `retained.rs` under the source budget; see the module docs there).
//! Counter pattern mirrors `vize_croquis::drawer::differential`.
//!
//! Compiled unconditionally (a few atomics, dead in production) so the
//! cfg-gated comparators in this crate *and* in `vize_atelier_vapor` can
//! record into one set of counters regardless of which crate's `test` cfg
//! or `davinci-differential` feature armed them.
//!
//! P1-9 adds the transform-lane *legacy* counters: every
//! `rewrite_expression` call that could not be served by the AST-driven
//! splice is classified by why, so the corpus run reports the
//! admitted/legacy split and the residual classes with measured numbers.

use core::sync::atomic::{AtomicU64, Ordering};

static TRANSFORM_REWRITE_COMPARISONS: AtomicU64 = AtomicU64::new(0);
static CODEGEN_REWRITE_COMPARISONS: AtomicU64 = AtomicU64::new(0);
static VAPOR_RESOLVE_COMPARISONS: AtomicU64 = AtomicU64::new(0);
static SHAPE_COMPARISONS: AtomicU64 = AtomicU64::new(0);

static TRANSFORM_REWRITE_LEGACY_PARAMS: AtomicU64 = AtomicU64::new(0);
static TRANSFORM_REWRITE_LEGACY_UNRETAINED: AtomicU64 = AtomicU64::new(0);
static TRANSFORM_REWRITE_LEGACY_DIALECT: AtomicU64 = AtomicU64::new(0);
static TRANSFORM_REWRITE_LEGACY_TS_STRIP: AtomicU64 = AtomicU64::new(0);

/// One retained-vs-legacy `rewrite_expression` output comparison passed.
pub fn record_transform_rewrite_comparison() {
    TRANSFORM_REWRITE_COMPARISONS.fetch_add(1, Ordering::Relaxed);
}

/// One retained-vs-legacy codegen prefixer output comparison passed.
pub fn record_codegen_rewrite_comparison() {
    CODEGEN_REWRITE_COMPARISONS.fetch_add(1, Ordering::Relaxed);
}

/// One retained-vs-legacy vapor resolve output comparison passed.
pub fn record_vapor_resolve_comparison() {
    VAPOR_RESOLVE_COMPARISONS.fetch_add(1, Ordering::Relaxed);
}

/// One retained-vs-legacy boolean shape-check comparison passed.
pub fn record_shape_comparison() {
    SHAPE_COMPARISONS.fetch_add(1, Ordering::Relaxed);
}

/// One `rewrite_expression` call stayed legacy: params position (the
/// synthesized-arrow validation path; retained ASTs never apply).
pub fn record_transform_rewrite_legacy_params() {
    TRANSFORM_REWRITE_LEGACY_PARAMS.fetch_add(1, Ordering::Relaxed);
}

/// One `rewrite_expression` call stayed legacy: no retained AST describes
/// the node's current bytes (never parsed whole per the P1-5 completeness
/// contract, guard-refused at the armature parse, or content rewritten
/// since parse).
pub fn record_transform_rewrite_legacy_unretained() {
    TRANSFORM_REWRITE_LEGACY_UNRETAINED.fetch_add(1, Ordering::Relaxed);
}

/// One `rewrite_expression` call stayed legacy: the retained AST failed the
/// `js_module_compatible` dialect gate.
pub fn record_transform_rewrite_legacy_dialect() {
    TRANSFORM_REWRITE_LEGACY_DIALECT.fetch_add(1, Ordering::Relaxed);
}

/// One `rewrite_expression` call stayed legacy: the TS strip rewrote the
/// bytes, so the retained AST no longer describes the rewritten text.
pub fn record_transform_rewrite_legacy_ts_strip() {
    TRANSFORM_REWRITE_LEGACY_TS_STRIP.fetch_add(1, Ordering::Relaxed);
}

/// Snapshot of the process-global monotone counters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DifferentialStats {
    /// Retained identifier-rewrite walks dual-run in the transform lane.
    pub transform_rewrite_comparisons: u64,
    /// Retained identifier-rewrite walks dual-run in the codegen lane.
    pub codegen_rewrite_comparisons: u64,
    /// Retained resolve walks dual-run in the vapor generate lane.
    pub vapor_resolve_comparisons: u64,
    /// Retained boolean shape checks dual-run against a legacy parse.
    pub shape_comparisons: u64,
}

/// Read the current counter snapshot.
pub fn stats() -> DifferentialStats {
    DifferentialStats {
        transform_rewrite_comparisons: TRANSFORM_REWRITE_COMPARISONS.load(Ordering::Relaxed),
        codegen_rewrite_comparisons: CODEGEN_REWRITE_COMPARISONS.load(Ordering::Relaxed),
        vapor_resolve_comparisons: VAPOR_RESOLVE_COMPARISONS.load(Ordering::Relaxed),
        shape_comparisons: SHAPE_COMPARISONS.load(Ordering::Relaxed),
    }
}

/// Snapshot of the transform-lane legacy (string-path) classification
/// counters (P1-9). Recorded only while the differential lane is armed, so
/// diffs of these across a run pair with [`DifferentialStats`]'s
/// `transform_rewrite_comparisons` (the admitted count) to give the
/// admitted/legacy split.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransformRewriteLegacyStats {
    /// Params-position calls (`as_params`): synthesized parameter lists.
    pub params: u64,
    /// No retained AST for the node's current bytes.
    pub unretained: u64,
    /// Retained AST rejected by the `js_module_compatible` dialect gate.
    pub dialect_rejected: u64,
    /// TS strip rewrote the bytes out from under the retained AST.
    pub ts_strip_rewrote: u64,
}

impl TransformRewriteLegacyStats {
    /// Total legacy-path `rewrite_expression` calls in the snapshot.
    pub fn total(&self) -> u64 {
        self.params + self.unretained + self.dialect_rejected + self.ts_strip_rewrote
    }
}

/// Read the current legacy-classification snapshot.
pub fn transform_rewrite_legacy_stats() -> TransformRewriteLegacyStats {
    TransformRewriteLegacyStats {
        params: TRANSFORM_REWRITE_LEGACY_PARAMS.load(Ordering::Relaxed),
        unretained: TRANSFORM_REWRITE_LEGACY_UNRETAINED.load(Ordering::Relaxed),
        dialect_rejected: TRANSFORM_REWRITE_LEGACY_DIALECT.load(Ordering::Relaxed),
        ts_strip_rewrote: TRANSFORM_REWRITE_LEGACY_TS_STRIP.load(Ordering::Relaxed),
    }
}
