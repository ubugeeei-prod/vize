//! Temporary template-walk counter (Davinci P2-12a).
//!
//! Counts the pre-S2 traversal work of the fused compile path: one **visit**
//! per template node a stage's descent dispatches on, and one **walk** per
//! stage traversal entered. The phase-2 exit gate compares the S2 pass
//! manager's traversal count against the numbers this probe recorded, so the
//! quantity has to be pinned before the work that could bias it - the P0-3
//! precedent ([`crate::expr_parse_probe`]), whose baseline lives in
//! `davinci-road/plan/expr-reparse-baseline.md` exactly the same way. This
//! probe's baseline is `davinci-road/plan/walk-baseline.md`, and
//! `budgets.toml [traversal]` gates it.
//!
//! # The counting rule
//!
//! A **visit** is counted where a stage's descent dispatches on a template
//! node's kind to decide how to continue. A **walk** is counted at a stage's
//! root entry only: recursion back into a dispatcher for a child's own
//! children continues the same walk. Both counters are per stage
//! ([`WalkStage`]), so a per-stage breakdown comes out of the same run that
//! produces the totals.
//!
//! # Instrumented sites
//!
//! | stage                     | visit sites                                                                                                                                                                                                    | walk site                              |
//! | ------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------- |
//! | [`WalkStage::Transform`]  | `lane::traverse::traverse_node`                                                                                                                                                                                | `lane::transform_inner`                |
//! | [`WalkStage::Codegen`]    | `codegen::node::generate_node`; `codegen::element::helpers::generate_root_node` (3 specialized arms); `codegen::element::v_once::generate_v_once_child` (2 specialized arms)                                    | `codegen::emit`                        |
//! | [`WalkStage::SsrCodegen`] | `codegen::helpers::process_child`; `codegen::element::vnode::vnode_child_expression`                                                                                                                           | `codegen::SsrCodegenContext::generate` |
//! | [`WalkStage::VaporLower`] | `lower::transform_children`; the `<template>` child loop in `transform::element`; the three child loops in `transform::element::deferred`; `transform::text::collect_text_runs`                                 | `lower::transform_to_ir_with_diagnostics` |
//!
//! The specialized-arm sites exist because those dispatchers fall through to
//! the stage's main funnel for the remaining variants, and counting the whole
//! function would double-count the fallthrough.
//!
//! # Two call shapes, and when each is correct
//!
//! Most sites use [`ssr_children`] / [`vapor_children`], which count inside
//! the iterator constructor so the count and the walk are the same event by
//! construction (the P0-3 shape) and instrumenting costs no extra source line.
//! [`record_visit`] / [`record_visits`] are used only where a dispatcher
//! branches on the whole list before reaching a loop — Vapor lowering's
//! combined-text case — because counting at the loop there would miss the
//! branch that bypasses it.
//!
//! # What is deliberately not counted
//!
//! Two classes, both named so the exclusion is reviewable rather than
//! implicit (the same list appears in `walk-baseline.md`):
//!
//! - **Subtree queries** - `codegen::generate::collect_helpers`,
//!   `codegen::slots::detect`, `codegen::element::helpers`'s namespace check,
//!   `codegen::generate::static_vnode`, `steps::hoist_static` (and its
//!   `static_type` classifier), `vize_atelier_vapor`'s
//!   `count_dynamic_element_children`, `is_static_element` and
//!   `generate_element_template`. These walk a subtree to answer a question or
//!   build a static string, not to run a stage over it.
//! - **Emission shortcuts** - the single-child inline in
//!   `codegen::children`, the single-child unwrap in `codegen::v_if::branch`,
//!   and the text concatenation in `codegen::slots::generate`. These consume a
//!   leaf child inside the parent's visit instead of descending into it, so
//!   the parent's visit already accounts for them.
//!
//! `vize_atelier_vapor::generate` walks Vapor IR rather than the template
//! tree, and `vize_croquis` walks the script AST; both are out of scope.
//!
//! The exclusions make this an approximation of total traversal work, pinned
//! exactly and reproducibly rather than estimated - which is what the phase
//! gate needs. P2-12b replaces it with the pass manager's own budget observer,
//! whose count is exact by construction, and this module is deleted with its
//! last call site.
//!
//! One relaxed atomic increment per event. The counters are process-global and
//! monotone - readers diff two loads around the region they care about, and a
//! recorder therefore runs as a single `#[test]` in its own binary.

use core::sync::atomic::{AtomicU64, Ordering};

/// A compile-path stage that walks the template tree.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(usize)]
pub enum WalkStage {
    /// The transform lane (`crate::lane`).
    Transform = 0,
    /// Base / DOM code generation (`crate::codegen`).
    Codegen = 1,
    /// SSR code generation (`vize_atelier_ssr::codegen`).
    SsrCodegen = 2,
    /// Vapor IR lowering (`vize_atelier_vapor::lower`).
    VaporLower = 3,
}

/// Every stage, in discriminant order, so a recorder can print a full
/// breakdown without hard-coding the list.
pub const WALK_STAGES: [WalkStage; 4] = [
    WalkStage::Transform,
    WalkStage::Codegen,
    WalkStage::SsrCodegen,
    WalkStage::VaporLower,
];

impl WalkStage {
    /// Stable identifier used in recorder output and in the baseline document.
    #[inline]
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            WalkStage::Transform => "transform",
            WalkStage::Codegen => "codegen",
            WalkStage::SsrCodegen => "ssr_codegen",
            WalkStage::VaporLower => "vapor_lower",
        }
    }
}

static VISITS: [AtomicU64; WALK_STAGES.len()] = [const { AtomicU64::new(0) }; WALK_STAGES.len()];
static WALKS: [AtomicU64; WALK_STAGES.len()] = [const { AtomicU64::new(0) }; WALK_STAGES.len()];

/// Count one template node visited by `stage`'s node dispatcher.
#[inline]
pub fn record_visit(stage: WalkStage) {
    VISITS[stage as usize].fetch_add(1, Ordering::Relaxed);
}

/// Count `nodes` template nodes visited by `stage`'s node dispatcher.
///
/// The bulk form exists for dispatchers that branch on a whole child list
/// before handling its members (Vapor lowering's combined-text case), where
/// counting at the list head is what makes the count branch-independent.
#[inline]
pub fn record_visits(stage: WalkStage, nodes: usize) {
    VISITS[stage as usize].fetch_add(nodes as u64, Ordering::Relaxed);
}

/// Iterate `children`, counting each as an [`SsrCodegen`](WalkStage::SsrCodegen)
/// visit.
///
/// Counting *inside the iterator constructor* is the P0-3 shape
/// ([`crate::expr_parse_probe::parse_arena`]): the count and the walk become
/// the same event by construction, so a loop cannot be added without being
/// counted. It also means instrumenting a dispatcher costs **no extra line**,
/// which is what keeps a temporary probe out of files that are already over
/// the 350-line source budget.
#[inline]
pub fn ssr_children<T>(children: &[T]) -> core::slice::Iter<'_, T> {
    record_visits(WalkStage::SsrCodegen, children.len());
    children.iter()
}

/// Iterate `children`, counting each as a
/// [`VaporLower`](WalkStage::VaporLower) visit. Same shape as
/// [`ssr_children`].
#[inline]
pub fn vapor_children<T>(children: &[T]) -> core::slice::Iter<'_, T> {
    record_visits(WalkStage::VaporLower, children.len());
    children.iter()
}

/// Count one traversal of the template tree entered by `stage`.
///
/// Recorded at the stage's root entry only: the recursion back into a
/// dispatcher for a child's own children continues the same walk.
#[inline]
pub fn record_walk(stage: WalkStage) {
    WALKS[stage as usize].fetch_add(1, Ordering::Relaxed);
}

/// Node visits counted for `stage` since process start (monotone).
#[inline]
#[must_use]
pub fn visit_count(stage: WalkStage) -> u64 {
    VISITS[stage as usize].load(Ordering::Relaxed)
}

/// Tree walks counted for `stage` since process start (monotone).
#[inline]
#[must_use]
pub fn walk_count(stage: WalkStage) -> u64 {
    WALKS[stage as usize].load(Ordering::Relaxed)
}

/// Node visits summed over every stage since process start (monotone).
///
/// This is the quantity `budgets.toml [traversal]` pins per fixture: the total
/// template-node visits one fused compile makes.
#[inline]
#[must_use]
pub fn total_visits() -> u64 {
    WALK_STAGES.iter().map(|stage| visit_count(*stage)).sum()
}

/// Tree walks summed over every stage since process start (monotone).
#[inline]
#[must_use]
pub fn total_walks() -> u64 {
    WALK_STAGES.iter().map(|stage| walk_count(*stage)).sum()
}

/// A snapshot of every counter, for diffing around one compile.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct WalkCounts {
    /// Per-stage node visits, indexed by [`WalkStage`] discriminant.
    pub visits: [u64; WALK_STAGES.len()],
    /// Per-stage tree walks, indexed by [`WalkStage`] discriminant.
    pub walks: [u64; WALK_STAGES.len()],
}

impl WalkCounts {
    /// Read every counter at once.
    #[must_use]
    pub fn snapshot() -> Self {
        let mut counts = Self::default();
        for stage in WALK_STAGES {
            counts.visits[stage as usize] = visit_count(stage);
            counts.walks[stage as usize] = walk_count(stage);
        }
        counts
    }

    /// The per-stage deltas from `earlier` to `self`.
    ///
    /// # Panics
    ///
    /// Panics if any counter in `earlier` exceeds its counterpart in `self`,
    /// which cannot happen for two ordered snapshots of monotone counters and
    /// therefore means the snapshots were taken out of order.
    #[must_use]
    pub fn since(self, earlier: Self) -> Self {
        let mut delta = Self::default();
        for stage in WALK_STAGES {
            let index = stage as usize;
            delta.visits[index] = self.visits[index]
                .checked_sub(earlier.visits[index])
                .expect("walk-probe snapshots are monotone and ordered");
            delta.walks[index] = self.walks[index]
                .checked_sub(earlier.walks[index])
                .expect("walk-probe snapshots are monotone and ordered");
        }
        delta
    }

    /// Node visits summed over every stage.
    #[must_use]
    pub fn total_visits(self) -> u64 {
        self.visits.iter().sum()
    }

    /// Tree walks summed over every stage.
    #[must_use]
    pub fn total_walks(self) -> u64 {
        self.walks.iter().sum()
    }
}
