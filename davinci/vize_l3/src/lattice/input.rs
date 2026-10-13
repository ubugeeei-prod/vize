use vize_l0::Span;

use super::{BindingId, BindingOrigin, EffectSet, EscapeKind, ReactivityClass};

/// Epistemic axis for a lattice classification.
///
/// One axis for every fact group: P4-6a moved the type into
/// `vize_l0::diag` so witnesses and the verifier share it, and the
/// lattice re-exports it with its folio spellings unchanged.
pub use vize_l0::diag::Verdict;

/// Summary provided by the retained-AST analyzer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BindingInput {
    pub id: BindingId,
    pub origin: BindingOrigin,
    pub effects: EffectSet,
    pub escape: EscapeKind,
    pub verdict: Verdict,
    pub span: Span,
}

impl BindingInput {
    /// A local binding with no effects or escape.
    #[must_use]
    pub const fn local(id: BindingId, span: Span) -> Self {
        Self {
            id,
            origin: BindingOrigin::Local,
            effects: EffectSet::empty(),
            escape: EscapeKind::None,
            verdict: Verdict::Proven,
            span,
        }
    }

    /// Override the binding origin.
    #[must_use]
    pub const fn with_origin(mut self, origin: BindingOrigin) -> Self {
        self.origin = origin;
        self
    }

    /// Override the effect set.
    #[must_use]
    pub const fn with_effects(mut self, effects: EffectSet) -> Self {
        self.effects = effects;
        self
    }

    /// Override the escape-analysis result.
    #[must_use]
    pub const fn with_escape(mut self, escape: EscapeKind) -> Self {
        self.escape = escape;
        self
    }

    /// Override the epistemic verdict.
    #[must_use]
    pub const fn with_verdict(mut self, verdict: Verdict) -> Self {
        self.verdict = verdict;
        self
    }
}

/// Published lattice fact for one binding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BindingFact {
    pub id: BindingId,
    pub class: ReactivityClass,
    pub verdict: Verdict,
    pub origin: BindingOrigin,
    pub effects: EffectSet,
    pub escape: EscapeKind,
    pub span: Span,
}

impl BindingFact {
    /// Returns the class only when the epistemic axis is proven.
    #[must_use]
    pub const fn proven_class(self) -> Option<ReactivityClass> {
        if self.verdict.is_proven() {
            Some(self.class)
        } else {
            None
        }
    }

    /// Consumer helper: facts never fire when their verdict is not proven.
    #[must_use]
    pub const fn fires_as(self, class: ReactivityClass) -> bool {
        self.verdict.is_proven() && self.class as u8 == class as u8
    }
}
