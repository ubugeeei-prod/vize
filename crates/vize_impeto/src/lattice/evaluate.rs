use vize_s0::{Allocator, Vec};

use super::{BindingFact, BindingId, BindingInput, ReactivityClass};

/// Reactivity-lattice facts for one S3 artifact.
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
    let class = input
        .origin
        .class_floor()
        .join(input.effects.class_floor())
        .join(input.escape.class_floor());
    BindingFact {
        id: input.id,
        class: provide_inject_cap(input.origin, class),
        verdict: input.verdict,
        origin: input.origin,
        effects: input.effects,
        escape: input.escape,
        span: input.span,
    }
}

const fn provide_inject_cap(
    origin: super::BindingOrigin,
    class: ReactivityClass,
) -> ReactivityClass {
    match origin {
        super::BindingOrigin::ProvideInject => class.join(ReactivityClass::Reactive),
        _ => class,
    }
}
