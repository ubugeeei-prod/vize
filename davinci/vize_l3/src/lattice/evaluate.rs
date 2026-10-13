use vize_l0::{Allocator, Vec};

use super::axis::classify;
use super::{BindingFact, BindingId, BindingInput};

/// Reactivity-lattice facts for one L3 artifact.
#[derive(Debug)]
pub struct LatticeFacts<'a> {
    pub bindings: Vec<'a, BindingFact>,
}

impl<'a> LatticeFacts<'a> {
    /// Empty fact group.
    #[must_use]
    pub fn new(allocator: &'a Allocator) -> Self {
        Self {
            bindings: Vec::new_in(&allocator),
        }
    }

    /// Add one binding fact.
    pub fn push(&mut self, fact: BindingFact) {
        self.bindings.push(fact);
    }

    /// Find a fact by id.
    #[must_use]
    pub fn get(&self, id: BindingId) -> Option<&BindingFact> {
        self.bindings.iter().find(|fact| fact.id == id)
    }
}

/// Evaluate all binding summaries into a lattice fact group.
#[must_use]
pub fn evaluate<'a>(
    allocator: &'a Allocator,
    inputs: impl IntoIterator<Item = BindingInput>,
) -> LatticeFacts<'a> {
    let mut facts = LatticeFacts::new(allocator);
    for input in inputs {
        facts.push(evaluate_binding(input));
    }
    facts
}

/// Naive declarative-rule evaluator for one binding summary.
#[must_use]
pub const fn evaluate_binding(input: BindingInput) -> BindingFact {
    BindingFact {
        id: input.id,
        class: classify(input.origin, input.effects, input.escape),
        verdict: input.verdict,
        origin: input.origin,
        effects: input.effects,
        escape: input.escape,
        span: input.span,
    }
}
