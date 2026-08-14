//! Davinci P1-6 differential lane — dual-run counters for the croquis
//! consumer migration (wave A).
//!
//! While croquis's expression-input layer migrates onto the parse-once
//! retained ASTs (P1-5, `SimpleExpressionNode::js_ast`), every migrated
//! read is dual-run against the legacy re-parse path for one release and
//! divergence panics at the migrated helper (never averaged, never
//! preferred silently). This module holds the process-global monotone
//! counters those comparators report through, so a lane run can prove how
//! much it actually compared.
//!
//! Compiled only under `cfg(any(test, feature = "davinci-differential"))`:
//!
//! - `cargo test -p vize_croquis` exercises the comparators in every unit
//!   test that walks a template through the drawer;
//! - `cargo test -p vize_croquis --features davinci-differential --test
//!   davinci_differential -- --nocapture` is the corpus-runnable entry
//!   (`VIZE_DAVINCI_DIFFERENTIAL_CORPUS=<dir>` widens the sweep from the
//!   committed fixture ladder to every `.vue` under `<dir>`);
//! - production builds (feature off, not test) compile none of this and
//!   run the retained path single-sided — zero cost when off.
//!
//! Lanes:
//!
//! - **identifier sets** — retained-AST walk vs legacy re-parse, asserted
//!   equal at `helpers/identifiers.rs`; both sides consume identical bytes
//!   (the retained walk only engages on comment-free text, so the
//!   comparison is parse-vs-walk over the same input, never a
//!   comment-stripping semantics question);
//! - **component references** — retained-AST shape check vs legacy local
//!   parse, asserted equal at `template/visit_element/second_pass.rs`;
//! - **v-for destructure shapes** — wave A forks no v-for path (the
//!   `helpers/v_for/oxc.rs` parses are binding-pattern parses over
//!   synthesized `let [...] = x` text, which no retained AST corresponds
//!   to), so this lane is a witness count of shapes computed while the
//!   lane is active; the fork-level dual-run arrives with P1-7/P1-8.

use core::sync::atomic::{AtomicU64, Ordering};

static IDENTIFIER_COMPARISONS: AtomicU64 = AtomicU64::new(0);
static COMPONENT_REFERENCE_COMPARISONS: AtomicU64 = AtomicU64::new(0);
static V_FOR_SHAPES: AtomicU64 = AtomicU64::new(0);

/// One retained-vs-legacy identifier-set comparison passed.
pub(crate) fn record_identifier_comparison() {
    IDENTIFIER_COMPARISONS.fetch_add(1, Ordering::Relaxed);
}

/// One retained-vs-legacy component-reference comparison passed.
pub(crate) fn record_component_reference_comparison() {
    COMPONENT_REFERENCE_COMPARISONS.fetch_add(1, Ordering::Relaxed);
}

/// One v-for destructure shape was computed while the lane was active.
pub(crate) fn record_v_for_shape() {
    V_FOR_SHAPES.fetch_add(1, Ordering::Relaxed);
}

/// Snapshot of the process-global differential counters (monotone; readers
/// diff two snapshots around the region they care about).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DifferentialStats {
    /// Retained-AST identifier walks dual-run against the legacy re-parse.
    pub identifier_comparisons: u64,
    /// Retained-AST component-reference checks dual-run against the legacy
    /// local parse.
    pub component_reference_comparisons: u64,
    /// v-for destructure shapes computed (witness count — see module docs).
    pub v_for_shapes: u64,
}

/// Read the current counter snapshot.
pub fn stats() -> DifferentialStats {
    DifferentialStats {
        identifier_comparisons: IDENTIFIER_COMPARISONS.load(Ordering::Relaxed),
        component_reference_comparisons: COMPONENT_REFERENCE_COMPARISONS.load(Ordering::Relaxed),
        v_for_shapes: V_FOR_SHAPES.load(Ordering::Relaxed),
    }
}
