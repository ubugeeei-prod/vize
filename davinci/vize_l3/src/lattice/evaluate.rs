use vize_l0::{Allocator, Vec};

use super::axis::classify;
use super::{BindingId, BindingInfo, BindingInput};

/// Reactivity-lattice facts for one L3 artifact.
#[derive(Debug)]
pub struct LatticeAnalysis<'a> {
    pub bindings: Vec<'a, BindingInfo>,
}

impl<'a> LatticeAnalysis<'a> {
    /// Empty fact group.
    #[must_use]
    pub fn new(allocator: &'a Allocator) -> Self {
        Self {
            bindings: Vec::new_in(&allocator),
        }
    }

    /// Add one binding fact.
    pub fn push(&mut self, fact: BindingInfo) {
        self.bindings.push(fact);
    }

    /// Find a fact by id.
    #[must_use]
    pub fn get(&self, id: BindingId) -> Option<&BindingInfo> {
        self.bindings.iter().find(|fact| fact.id == id)
    }
}

/// Evaluate all binding summaries into a lattice fact group.
#[must_use]
pub fn evaluate<'a>(
    allocator: &'a Allocator,
    inputs: impl IntoIterator<Item = BindingInput>,
) -> LatticeAnalysis<'a> {
    let mut facts = LatticeAnalysis::new(allocator);
    for input in inputs {
        facts.push(evaluate_binding(input));
    }
    facts
}

/// Naive declarative-rule evaluator for one binding summary.
#[must_use]
pub const fn evaluate_binding(input: BindingInput) -> BindingInfo {
    BindingInfo {
        id: input.id,
        class: classify(input.origin, input.effects, input.escape),
        verdict: input.verdict,
        origin: input.origin,
        effects: input.effects,
        escape: input.escape,
        span: input.span,
    }
}
