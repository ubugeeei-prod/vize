use super::{EffectId, OpId};

/// State-edge classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum EdgeKind {
    DomOrder = 0,
    EffectOrder = 1,
    DataDependency = 2,
}

impl EdgeKind {
    /// Stable folio spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::DomOrder => "dom-order",
            Self::EffectOrder => "effect-order",
            Self::DataDependency => "data-dependency",
        }
    }

    /// Parse the stable folio spelling.
    #[must_use]
    pub const fn from_str(value: &str) -> Option<Self> {
        match value.as_bytes() {
            b"dom-order" => Some(Self::DomOrder),
            b"effect-order" => Some(Self::EffectOrder),
            b"data-dependency" => Some(Self::DataDependency),
            _ => None,
        }
    }
}

/// An explicit ordering or dependency edge between ops.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StateEdge {
    pub from: OpId,
    pub to: OpId,
    pub kind: EdgeKind,
    pub effect: Option<EffectId>,
}

impl StateEdge {
    /// A stage-wide edge.
    #[must_use]
    pub const fn new(from: OpId, to: OpId, kind: EdgeKind) -> Self {
        Self {
            from,
            to,
            kind,
            effect: None,
        }
    }

    /// An edge scoped to one effect.
    #[must_use]
    pub const fn scoped(from: OpId, to: OpId, kind: EdgeKind, effect: EffectId) -> Self {
        Self {
            from,
            to,
            kind,
            effect: Some(effect),
        }
    }
}
