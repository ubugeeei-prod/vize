//! The Impeto op family: flat, id-addressed operations plus explicit state
//! edges.
//!
//! S3 deliberately does not reuse Vapor's nested block IR. Regions name
//! containment, effects name update scopes, and state edges name ordering.
//! Later lowering can therefore ask graph questions without trusting an
//! incidental traversal order.

use core::fmt;

use vize_s0::{Allocator, Span, Vec};

/// One operation id in a [`Program`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OpId(u32);

impl OpId {
    /// Create an operation id from its numeric index.
    #[must_use]
    pub const fn new(index: u32) -> Self {
        Self(index)
    }

    /// Numeric index.
    #[must_use]
    pub const fn index(self) -> u32 {
        self.0
    }
}

impl fmt::Display for OpId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "op#{}", self.0)
    }
}

/// One region id. Region `0` is the required root region.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RegionId(u32);

impl RegionId {
    /// The root region every program must define.
    pub const ROOT: Self = Self(0);

    /// Create a region id from its numeric index.
    #[must_use]
    pub const fn new(index: u32) -> Self {
        Self(index)
    }

    /// Numeric index.
    #[must_use]
    pub const fn index(self) -> u32 {
        self.0
    }
}

impl fmt::Display for RegionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "r#{}", self.0)
    }
}

/// One effect-scope id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EffectId(u32);

impl EffectId {
    /// Create an effect-scope id from its numeric index.
    #[must_use]
    pub const fn new(index: u32) -> Self {
        Self(index)
    }

    /// Numeric index.
    #[must_use]
    pub const fn index(self) -> u32 {
        self.0
    }
}

impl fmt::Display for EffectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "fx#{}", self.0)
    }
}

/// The named phase an Impeto artifact currently satisfies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Phase {
    /// S2 has lowered into flat S3 ops; partition and schedule are not fixed.
    Built,
    /// Static/dynamic partition facts have been attached.
    Partitioned,
    /// State edges are in execution order.
    Scheduled,
}

impl Phase {
    /// Stable folio spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Built => "built",
            Self::Partitioned => "partitioned",
            Self::Scheduled => "scheduled",
        }
    }

    /// Parse the stable folio spelling.
    #[must_use]
    pub const fn from_str(value: &str) -> Option<Self> {
        match value.as_bytes() {
            b"built" => Some(Self::Built),
            b"partitioned" => Some(Self::Partitioned),
            b"scheduled" => Some(Self::Scheduled),
            _ => None,
        }
    }
}

/// The 16 operation kinds generalized from Vapor's current flat operation set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum OpKind {
    SetProp = 0,
    SetDynamicProps = 1,
    SetText = 2,
    SetEvent = 3,
    SetHtml = 4,
    SetTemplateRef = 5,
    InsertNode = 6,
    PrependNode = 7,
    Directive = 8,
    If = 9,
    For = 10,
    CreateComponent = 11,
    SlotOutlet = 12,
    GetTextChild = 13,
    ChildRef = 14,
    NextRef = 15,
}

impl OpKind {
    /// Stable stage-wide mnemonic.
    #[must_use]
    pub const fn mnemonic(self) -> &'static str {
        match self {
            Self::SetProp => "impeto.set-prop",
            Self::SetDynamicProps => "impeto.set-dynamic-props",
            Self::SetText => "impeto.set-text",
            Self::SetEvent => "impeto.set-event",
            Self::SetHtml => "impeto.set-html",
            Self::SetTemplateRef => "impeto.set-template-ref",
            Self::InsertNode => "impeto.insert-node",
            Self::PrependNode => "impeto.prepend-node",
            Self::Directive => "impeto.directive",
            Self::If => "impeto.if",
            Self::For => "impeto.for",
            Self::CreateComponent => "impeto.create-component",
            Self::SlotOutlet => "impeto.slot-outlet",
            Self::GetTextChild => "impeto.get-text-child",
            Self::ChildRef => "impeto.child-ref",
            Self::NextRef => "impeto.next-ref",
        }
    }

    /// Parse a stable stage-wide mnemonic.
    #[must_use]
    pub const fn from_mnemonic(value: &str) -> Option<Self> {
        match value.as_bytes() {
            b"impeto.set-prop" => Some(Self::SetProp),
            b"impeto.set-dynamic-props" => Some(Self::SetDynamicProps),
            b"impeto.set-text" => Some(Self::SetText),
            b"impeto.set-event" => Some(Self::SetEvent),
            b"impeto.set-html" => Some(Self::SetHtml),
            b"impeto.set-template-ref" => Some(Self::SetTemplateRef),
            b"impeto.insert-node" => Some(Self::InsertNode),
            b"impeto.prepend-node" => Some(Self::PrependNode),
            b"impeto.directive" => Some(Self::Directive),
            b"impeto.if" => Some(Self::If),
            b"impeto.for" => Some(Self::For),
            b"impeto.create-component" => Some(Self::CreateComponent),
            b"impeto.slot-outlet" => Some(Self::SlotOutlet),
            b"impeto.get-text-child" => Some(Self::GetTextChild),
            b"impeto.child-ref" => Some(Self::ChildRef),
            b"impeto.next-ref" => Some(Self::NextRef),
            _ => None,
        }
    }
}

/// Region metadata for the flat S3 graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Region {
    pub id: RegionId,
    pub parent: Option<RegionId>,
    pub owner: Option<OpId>,
    pub span: Span,
}

impl Region {
    /// Root region for a program.
    #[must_use]
    pub const fn root(span: Span) -> Self {
        Self {
            id: RegionId::ROOT,
            parent: None,
            owner: None,
            span,
        }
    }

    /// A nested region owned by an op in `parent`.
    #[must_use]
    pub const fn child(id: RegionId, parent: RegionId, owner: OpId, span: Span) -> Self {
        Self {
            id,
            parent: Some(parent),
            owner: Some(owner),
            span,
        }
    }
}

/// One flat Impeto operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Op {
    pub id: OpId,
    pub kind: OpKind,
    pub region: RegionId,
    pub effect: Option<EffectId>,
    pub span: Span,
}

impl Op {
    /// A new op in `region`.
    #[must_use]
    pub const fn new(id: OpId, kind: OpKind, region: RegionId, span: Span) -> Self {
        Self {
            id,
            kind,
            region,
            effect: None,
            span,
        }
    }

    /// Attach the op to an effect scope.
    #[must_use]
    pub const fn with_effect(mut self, effect: EffectId) -> Self {
        self.effect = Some(effect);
        self
    }
}

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

/// Effect-scope metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EffectScope {
    pub id: EffectId,
    pub owner: OpId,
    pub region: RegionId,
    pub span: Span,
}

/// One S3 program artifact.
#[derive(Debug)]
pub struct Program<'a> {
    pub phase: Phase,
    pub regions: Vec<'a, Region>,
    pub ops: Vec<'a, Op>,
    pub edges: Vec<'a, StateEdge>,
    pub effects: Vec<'a, EffectScope>,
}

impl<'a> Program<'a> {
    /// Empty program for `phase`.
    #[must_use]
    pub fn new(allocator: &'a Allocator, phase: Phase) -> Self {
        Self {
            phase,
            regions: Vec::new_in(&allocator),
            ops: Vec::new_in(&allocator),
            edges: Vec::new_in(&allocator),
            effects: Vec::new_in(&allocator),
        }
    }

    pub fn push_region(&mut self, region: Region) {
        self.regions.push(region);
    }

    pub fn push_op(&mut self, op: Op) {
        self.ops.push(op);
    }

    pub fn push_edge(&mut self, edge: StateEdge) {
        self.edges.push(edge);
    }

    pub fn push_effect(&mut self, effect: EffectScope) {
        self.effects.push(effect);
    }
}

const _: () = assert!(!core::mem::needs_drop::<Op>());
const _: () = assert!(!core::mem::needs_drop::<Region>());
const _: () = assert!(!core::mem::needs_drop::<StateEdge>());
const _: () = assert!(!core::mem::needs_drop::<EffectScope>());
const _: () = assert!(!core::mem::needs_drop::<Program<'static>>());

#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<Op>() <= 32);
    assert!(core::mem::size_of::<Region>() <= 32);
    assert!(core::mem::size_of::<StateEdge>() <= 24);
    assert!(core::mem::size_of::<EffectScope>() <= 24);
};
