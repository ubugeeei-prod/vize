use vize_s0::Span;

use super::{BindingId, EffectSet, ReactivityClass};

/// Where the binding value enters the component.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BindingOrigin {
    Local,
    Prop,
    ProvideInject,
    TemplateRef,
}

impl BindingOrigin {
    /// Stable folio spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Prop => "prop",
            Self::ProvideInject => "provide-inject",
            Self::TemplateRef => "template-ref",
        }
    }

    /// Parse the stable folio spelling.
    #[must_use]
    pub const fn from_str(value: &str) -> Option<Self> {
        match value.as_bytes() {
            b"local" => Some(Self::Local),
            b"prop" => Some(Self::Prop),
            b"provide-inject" => Some(Self::ProvideInject),
            b"template-ref" => Some(Self::TemplateRef),
            _ => None,
        }
    }

    /// The best possible class for this origin.
    #[must_use]
    pub const fn class_floor(self) -> ReactivityClass {
        match self {
            Self::Local => ReactivityClass::Static,
            Self::Prop => ReactivityClass::PropsStable,
            Self::ProvideInject | Self::TemplateRef => ReactivityClass::Reactive,
        }
    }
}

/// Escape-analysis result for one binding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EscapeKind {
    None,
    Returned,
    Stored,
    Global,
}

impl EscapeKind {
    /// Stable folio spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Returned => "returned",
            Self::Stored => "stored",
            Self::Global => "global",
        }
    }

    /// Parse the stable folio spelling.
    #[must_use]
    pub const fn from_str(value: &str) -> Option<Self> {
        match value.as_bytes() {
            b"none" => Some(Self::None),
            b"returned" => Some(Self::Returned),
            b"stored" => Some(Self::Stored),
            b"global" => Some(Self::Global),
            _ => None,
        }
    }

    /// The lattice floor implied by escape analysis.
    #[must_use]
    pub const fn class_floor(self) -> ReactivityClass {
        match self {
            Self::None => ReactivityClass::Static,
            Self::Returned => ReactivityClass::PropsStable,
            Self::Stored => ReactivityClass::Reactive,
            Self::Global => ReactivityClass::Unstable,
        }
    }
}

/// Epistemic axis for a lattice classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Verdict {
    Proven,
    Refuted,
    Unknown,
}

impl Verdict {
    /// Stable folio spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Proven => "proven",
            Self::Refuted => "refuted",
            Self::Unknown => "unknown",
        }
    }

    /// Parse the stable folio spelling.
    #[must_use]
    pub const fn from_str(value: &str) -> Option<Self> {
        match value.as_bytes() {
            b"proven" => Some(Self::Proven),
            b"refuted" => Some(Self::Refuted),
            b"unknown" => Some(Self::Unknown),
            _ => None,
        }
    }

    /// Whether a consumer may use the value as proof.
    #[must_use]
    pub const fn is_proven(self) -> bool {
        matches!(self, Self::Proven)
    }
}

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
