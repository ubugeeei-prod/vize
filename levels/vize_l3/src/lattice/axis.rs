use super::{EffectSet, ReactivityClass};

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

/// The naive declarative-rule evaluator on the value axis: join the floor
/// every premise demands, then apply the provide/inject cap.
///
/// It reads nothing but the three axes, so the Kani harness in
/// `davinci/vize_l3/src/lattice/kani.rs` verifies this exact source against the
/// rule table.
#[must_use]
pub const fn classify(
    origin: BindingOrigin,
    effects: EffectSet,
    escape: EscapeKind,
) -> ReactivityClass {
    let class = origin
        .class_floor()
        .join(effects.class_floor())
        .join(escape.class_floor());
    match origin {
        BindingOrigin::ProvideInject => class.join(ReactivityClass::Reactive),
        _ => class,
    }
}
