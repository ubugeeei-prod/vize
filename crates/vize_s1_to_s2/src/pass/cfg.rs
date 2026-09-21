//! The template-complexity pass (Davinci P4-9a): cyclomatic and cognitive
//! complexity of one component's template, computed over the S2 control
//! regions instead of scanned from expression text.
//!
//! The metric definition is `davinci-road/plan/complexity-metrics.md`; this
//! module is its production implementation and the TS-34 naive evaluator
//! (`tests/cfg_complexity_oracle.rs`) is the independent one.
//!
//! # The control-flow graph, and why it is not materialized
//!
//! S2 control is structured: a `ui.if` owns one region per branch, a
//! `ui.for` owns its repeated region, and nothing jumps. For a
//! single-entry, single-exit structured graph McCabe's `E − N + 2` equals
//! `1 +` the number of binary decisions, so the pass counts decisions over
//! the regions and never allocates the graph. The oracle builds the graph
//! explicitly and computes `E − N + 2` from it, which is what makes the
//! agreement a check of the equivalence rather than of one formula twice.
//!
//! # Classification (the review point)
//!
//! **`Optional`, `Fusable`, `Preserved::ALL`** — see [`DESC`], pinned in
//! `pass.rs`.
//!
//! - *Why optional:* no emitter reads the product. Skipping the pass loses
//!   the lint and Doctor inputs and nothing a compiled artifact can
//!   observe; the DOM product path never plans it.
//! - *Why fusable:* every contribution is computed from the op being
//!   visited, the nesting depth inherited from its enclosing regions and
//!   the op's own expressions — an inherited attribute plus a local
//!   synthesized sum, one pre-order visit, no sibling lookahead, no
//!   fixpoint. In the transform plan it joins `hoist-static`'s group, so
//!   the plan gains a pass and no walk.
//! - `Preserved::ALL`: the pass borrows the artifact immutably; it writes
//!   no provenance and no diagnostic.
//!
//! # Accounting
//!
//! The recursion mints through the shared [`super::walk::PageWalk`], so
//! every contribution names the page-order id of the op that carries it,
//! and the run ends at the same accounting assertion every pass shares.

use alloc::vec::Vec as StdVec;

use vize_davinci::id::NodeId;
use vize_davinci::pass::{Fusability, PassDesc, PassKind, Preserved};
use vize_s0::Span;

use super::walk::{PageWalk, assert_accounting};
use crate::lower::Lowered;

mod expr;
mod print;
mod regions;

pub use print::print_facts;

/// The pass name in pipeline strings and folio pages.
pub const NAME: &str = "template-complexity";

/// The pass description — classification reasoning in the module docs.
pub const DESC: PassDesc = PassDesc::new(
    NAME,
    PassKind::Optional,
    Fusability::Fusable,
    // Pure analysis over a shared borrow: nothing moves.
    Preserved::ALL,
);

/// Where one increment comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DecisionKind {
    /// A `v-if` condition: one decision, a structure (+1 + nesting).
    If,
    /// A `v-else-if` condition: one decision, a flat +1.
    ElseIf,
    /// The `v-else` branch: no decision, a flat +1.
    Else,
    /// A `v-for` repetition: one decision, a structure (+1 + nesting).
    For,
    /// A scoped-slot region: no increment of its own; it deepens the
    /// nesting of everything inside it (a render callback, the lambda
    /// case of the cognitive model).
    ScopedSlot,
    /// One maximal tree of `&&` / `||` / `??` in an expression: one
    /// decision per operator, +1 per run of like operators.
    Logical,
    /// One `?:`: one decision, a structure (+1 + nesting).
    Conditional,
    /// An evaluated expression position without a retained AST (opaque,
    /// foreign, or a Vue 2 filter chain): adds nothing, counted.
    Unknown,
}

impl DecisionKind {
    /// The kind's spelling in printed breakdowns and JSON reports.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::If => "v-if",
            Self::ElseIf => "v-else-if",
            Self::Else => "v-else",
            Self::For => "v-for",
            Self::ScopedSlot => "scoped-slot",
            Self::Logical => "logical",
            Self::Conditional => "conditional",
            Self::Unknown => "unknown",
        }
    }
}

/// One entry of a component's complexity breakdown.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Contribution {
    /// The authored range the increment points at (a condition, a `v-for`
    /// source, a branch, an operator tree, a `?:`).
    pub span: Span,
    /// What kind of construct this is.
    pub kind: DecisionKind,
    /// Page-order id of the op carrying the construct (`None` only past
    /// the id space's saturation, mirroring the lowering).
    pub op: Option<NodeId>,
    /// Enclosing nesting regions (`ui.if` branches, `ui.for` bodies,
    /// scoped-slot bodies, and — inside an expression — enclosing `?:`).
    pub nesting: u32,
    /// Cyclomatic increment.
    pub cyclomatic: u32,
    /// Cognitive increment.
    pub cognitive: u32,
}

/// One component's own template complexity: the P4-9a fact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComplexityFacts {
    /// `1 +` every contribution's cyclomatic increment.
    pub cyclomatic: u32,
    /// The sum of every contribution's cognitive increment.
    pub cognitive: u32,
    /// Evaluated expression positions without a retained AST.
    pub unknown: u32,
    /// The deepest nesting-region depth the template reaches.
    pub max_nesting: u32,
    /// The breakdown, in [`Contribution::sort_key`] order.
    pub contributions: StdVec<Contribution>,
}

impl Contribution {
    /// The breakdown order: source position, then kind (a `v-if` before
    /// the operator tree of its own condition, which shares its span).
    #[must_use]
    pub const fn sort_key(&self) -> (u32, u32, DecisionKind) {
        (self.span.start, self.span.end, self.kind)
    }
}

/// Facts cross compile boundaries with their artifact (P1-11).
const _: () = {
    const fn assert_owned<T: 'static>() {}
    assert_owned::<ComplexityFacts>();
};

/// 64-bit footprint of one breakdown row.
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<Contribution>() == 28);

impl ComplexityFacts {
    /// Facts of an empty template: one path, nothing else.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            cyclomatic: 1,
            cognitive: 0,
            unknown: 0,
            max_nesting: 0,
            contributions: StdVec::new(),
        }
    }

    pub(super) fn push(&mut self, contribution: Contribution) {
        self.cyclomatic = self.cyclomatic.saturating_add(contribution.cyclomatic);
        self.cognitive = self.cognitive.saturating_add(contribution.cognitive);
        if contribution.kind == DecisionKind::Unknown {
            self.unknown = self.unknown.saturating_add(1);
        }
        self.contributions.push(contribution);
    }

    /// Fill in a row pushed before its increments were known (an operator
    /// tree is counted while its operands are being scored).
    pub(super) fn settle(&mut self, index: usize, cyclomatic: u32, cognitive: u32) {
        if let Some(row) = self.contributions.get_mut(index) {
            row.cyclomatic = cyclomatic;
            row.cognitive = cognitive;
            self.cyclomatic = self.cyclomatic.saturating_add(cyclomatic);
            self.cognitive = self.cognitive.saturating_add(cognitive);
        }
    }
}

/// Run the pass over `lowered`.
///
/// One pre-order walk in page order over a shared borrow.
///
/// # Panics
///
/// Panics only on broken id accounting — a compiler bug by the id law,
/// never an input property.
#[must_use]
pub fn run(lowered: &Lowered<'_>) -> ComplexityFacts {
    let mut facts = ComplexityFacts::empty();
    let mut walk = PageWalk::new();
    regions::visit_region(&mut walk, &lowered.root.ops, 0, &mut facts);
    assert_accounting(&walk, lowered.op_count, NAME);
    facts
        .contributions
        .sort_unstable_by_key(Contribution::sort_key);
    facts
}
