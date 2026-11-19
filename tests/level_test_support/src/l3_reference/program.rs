//! The L3 graph as its committed dump page spells it. Written from the page
//! format, not from `vize_l3`, and free of dependencies so Kani includes it.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Built,
    Partitioned,
    Scheduled,
}

impl Phase {
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "built" => Some(Self::Built),
            "partitioned" => Some(Self::Partitioned),
            "scheduled" => Some(Self::Scheduled),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpKind {
    SetProp,
    SetDynamicProps,
    SetText,
    SetEvent,
    SetHtml,
    SetTemplateRef,
    InsertNode,
    PrependNode,
    Directive,
    Branch,
    Loop,
    CreateComponent,
    SlotOutlet,
    GetTextChild,
    ChildRef,
    NextRef,
    SetChildValue,
}

impl OpKind {
    pub const ALL: [Self; 17] = [
        Self::SetProp,
        Self::SetDynamicProps,
        Self::SetText,
        Self::SetEvent,
        Self::SetHtml,
        Self::SetTemplateRef,
        Self::InsertNode,
        Self::PrependNode,
        Self::Directive,
        Self::Branch,
        Self::Loop,
        Self::CreateComponent,
        Self::SlotOutlet,
        Self::GetTextChild,
        Self::ChildRef,
        Self::NextRef,
        Self::SetChildValue,
    ];

    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        Some(match text {
            "l3.set-prop" => Self::SetProp,
            "l3.set-dynamic-props" => Self::SetDynamicProps,
            "l3.set-text" => Self::SetText,
            "l3.set-event" => Self::SetEvent,
            "l3.set-html" => Self::SetHtml,
            "l3.set-template-ref" => Self::SetTemplateRef,
            "l3.insert-node" => Self::InsertNode,
            "l3.prepend-node" => Self::PrependNode,
            "l3.directive" => Self::Directive,
            "l3.if" => Self::Branch,
            "l3.for" => Self::Loop,
            "l3.create-component" => Self::CreateComponent,
            "l3.slot-outlet" => Self::SlotOutlet,
            "l3.get-text-child" => Self::GetTextChild,
            "l3.child-ref" => Self::ChildRef,
            "l3.next-ref" => Self::NextRef,
            "l3.set-child-value" => Self::SetChildValue,
            _ => return None,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EdgeKind {
    DomOrder,
    EffectOrder,
    DataDependency,
}

impl EdgeKind {
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "dom-order" => Some(Self::DomOrder),
            "effect-order" => Some(Self::EffectOrder),
            "data-dependency" => Some(Self::DataDependency),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Span {
    pub start: u32,
    pub stop: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Region {
    pub id: u32,
    pub parent: Option<u32>,
    pub owner: Option<u32>,
    pub span: Span,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Op {
    pub id: u32,
    pub kind: OpKind,
    pub region: u32,
    pub effect: Option<u32>,
    pub span: Span,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StateEdge {
    pub source: u32,
    pub target: u32,
    pub kind: EdgeKind,
    pub effect: Option<u32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EffectScope {
    pub id: u32,
    pub owner: u32,
    pub region: u32,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Program {
    pub phase: Phase,
    pub regions: Vec<Region>,
    pub ops: Vec<Op>,
    pub edges: Vec<StateEdge>,
    pub effects: Vec<EffectScope>,
}

impl Program {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            phase: Phase::Built,
            regions: Vec::new(),
            ops: Vec::new(),
            edges: Vec::new(),
            effects: Vec::new(),
        }
    }

    /// Op ids in list order.
    #[must_use]
    pub fn ids(&self) -> Vec<u32> {
        self.ops.iter().map(|op| op.id).collect()
    }

    /// The first region with `id`, as every lookup of the reference resolves it.
    #[must_use]
    pub fn region(&self, id: u32) -> Option<&Region> {
        self.regions.iter().find(|region| region.id == id)
    }

    #[must_use]
    pub fn op(&self, id: u32) -> Option<&Op> {
        self.ops.iter().find(|op| op.id == id)
    }

    #[must_use]
    pub fn effect(&self, id: u32) -> Option<&EffectScope> {
        self.effects.iter().find(|effect| effect.id == id)
    }
}
