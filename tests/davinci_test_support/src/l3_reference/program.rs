//! The L3 graph as its committed dump page spells it. Written from the page
//! format, not from `vize_l3`, and free of dependencies so the Creusot proof
//! crate (`creusot.rs`) includes it.
//!
//! Under `cfg(creusot)` the derives the laws never read (`Debug`, and the
//! discriminant-reading `PartialEq`/`Eq`) and the string parsers are left out
//! of the proof crate: the folio parser runs them on every committed graph.

#[cfg(creusot)]
use creusot_std::prelude::Clone;
#[cfg(creusot)]
use creusot_std::prelude::*;

#[cfg_attr(not(creusot), derive(Clone, Copy, Debug, PartialEq, Eq))]
#[cfg_attr(creusot, derive(Clone, Copy))]
pub enum Phase {
    Built,
    Partitioned,
    Scheduled,
}

impl Phase {
    #[cfg(not(creusot))]
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

#[cfg_attr(not(creusot), derive(Clone, Copy, Debug, PartialEq, Eq))]
#[cfg_attr(creusot, derive(Clone, Copy))]
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

    #[cfg(not(creusot))]
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

#[cfg_attr(not(creusot), derive(Clone, Copy, Debug, PartialEq, Eq))]
#[cfg_attr(creusot, derive(Clone, Copy))]
pub enum EdgeKind {
    DomOrder,
    EffectOrder,
    DataDependency,
}

impl EdgeKind {
    #[cfg(not(creusot))]
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

#[cfg_attr(not(creusot), derive(Clone, Copy, Debug, PartialEq, Eq))]
#[cfg_attr(creusot, derive(Clone, Copy))]
pub struct Span {
    pub start: u32,
    pub stop: u32,
}

#[cfg_attr(not(creusot), derive(Clone, Copy, Debug, PartialEq, Eq))]
#[cfg_attr(creusot, derive(Clone, Copy))]
pub struct Region {
    pub id: u32,
    pub parent: Option<u32>,
    pub owner: Option<u32>,
    pub span: Span,
}

#[cfg_attr(not(creusot), derive(Clone, Copy, Debug, PartialEq, Eq))]
#[cfg_attr(creusot, derive(Clone, Copy))]
pub struct Op {
    pub id: u32,
    pub kind: OpKind,
    pub region: u32,
    pub effect: Option<u32>,
    pub span: Span,
}

#[cfg_attr(not(creusot), derive(Clone, Copy, Debug, PartialEq, Eq))]
#[cfg_attr(creusot, derive(Clone, Copy))]
pub struct StateEdge {
    pub source: u32,
    pub target: u32,
    pub kind: EdgeKind,
    pub effect: Option<u32>,
}

#[cfg_attr(not(creusot), derive(Clone, Copy, Debug, PartialEq, Eq))]
#[cfg_attr(creusot, derive(Clone, Copy))]
pub struct EffectScope {
    pub id: u32,
    pub owner: u32,
    pub region: u32,
    pub span: Span,
}

#[cfg_attr(not(creusot), derive(Clone, Debug, PartialEq, Eq))]
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
    #[cfg_attr(creusot, ensures(result@.len() == self.ops@.len()))]
    #[cfg_attr(creusot, ensures(forall<i: Int> 0 <= i && i < result@.len()
        ==> result@[i] == self.ops@[i].id))]
    #[must_use]
    pub fn ids(&self) -> Vec<u32> {
        let mut ids = Vec::new();
        #[cfg_attr(creusot, invariant(ids@.len() == produced.len()))]
        #[cfg_attr(creusot, invariant(forall<i: Int> 0 <= i && i < produced.len()
            ==> ids@[i] == self.ops@[i].id))]
        for op in &self.ops {
            ids.push(op.id);
        }
        ids
    }

    /// The first region with `id`, as every lookup of the reference resolves it.
    #[must_use]
    pub fn region(&self, id: u32) -> Option<&Region> {
        lookup(&self.regions, id)
    }

    #[must_use]
    pub fn op(&self, id: u32) -> Option<&Op> {
        lookup(&self.ops, id)
    }

    #[must_use]
    pub fn effect(&self, id: u32) -> Option<&EffectScope> {
        lookup(&self.effects, id)
    }
}

/// A table row that lookups find by its id.
pub trait Row {
    /// The id, as the specifications read it.
    #[cfg(creusot)]
    #[logic]
    fn key_logic(self) -> u32;

    #[cfg_attr(creusot, ensures(result == (*self).key_logic()))]
    fn key(&self) -> u32;
}

/// A bare id is its own key: `index_of` over a list of ids.
impl Row for u32 {
    #[cfg(creusot)]
    #[logic(open)]
    fn key_logic(self) -> u32 {
        self
    }

    #[cfg_attr(creusot, ensures(result == (*self).key_logic()))]
    fn key(&self) -> u32 {
        *self
    }
}

macro_rules! row_by_id {
    ($($row:ty),*) => {$(
        impl Row for $row {
            #[cfg(creusot)]
            #[logic(open)]
            fn key_logic(self) -> u32 {
                self.id
            }

            #[cfg_attr(creusot, ensures(result == (*self).key_logic()))]
            fn key(&self) -> u32 {
                self.id
            }
        }
    )*};
}

row_by_id!(Region, Op, EffectScope);

/// Position `i` holds the first row with `id` (`List.indexOf?` is `some i`).
#[cfg(creusot)]
#[logic(open)]
pub fn first_at<T: Row>(rows: Seq<T>, id: u32, i: Int) -> bool {
    pearlite! {
        0 <= i && i < rows.len() && rows[i].key_logic() == id
            && forall<j: Int> 0 <= j && j < i ==> rows[j].key_logic() != id
    }
}

/// No row has `id` (`List.indexOf?` is `none`).
#[cfg(creusot)]
#[logic(open)]
pub fn absent<T: Row>(rows: Seq<T>, id: u32) -> bool {
    pearlite! { forall<j: Int> 0 <= j && j < rows.len() ==> rows[j].key_logic() != id }
}

/// Some row has `id`, and so a first one.
#[cfg(creusot)]
#[logic(open)]
pub fn found<T: Row>(rows: Seq<T>, id: u32) -> bool {
    pearlite! { exists<i: Int> first_at(rows, id, i) }
}

/// The first `n` rows have distinct ids.
#[cfg(creusot)]
#[logic(open)]
pub fn distinct_before<T: Row>(rows: Seq<T>, n: Int) -> bool {
    pearlite! {
        forall<i: Int, j: Int> 0 <= i && i < j && j < n
            ==> rows[i].key_logic() != rows[j].key_logic()
    }
}

/// `List.Nodup` of the ids: no two positions share one.
#[cfg(creusot)]
#[logic(open)]
pub fn distinct<T: Row>(rows: Seq<T>) -> bool {
    pearlite! { distinct_before(rows, rows.len()) }
}

/// The position of the first row with `id`, as the validator's `op_index`
/// computes it. For distinct ids it is the position of the id.
#[cfg_attr(creusot, ensures(match result {
    Some(i) => first_at(rows@, id, i@)
        && forall<j: Int> first_at(rows@, id, j) ==> j == i@,
    None => absent(rows@, id) && forall<j: Int> !first_at(rows@, id, j),
}))]
#[cfg_attr(creusot, ensures(forall<k: Int> distinct(rows@) && 0 <= k && k < rows@.len()
    && rows@[k].key_logic() == id ==> match result { Some(i) => i@ == k, None => false }))]
#[must_use]
pub fn index_of<T: Row>(rows: &[T], id: u32) -> Option<usize> {
    #[cfg_attr(creusot, invariant(forall<j: Int> 0 <= j && j < produced.len()
        ==> rows@[j].key_logic() != id))]
    for i in 0..rows.len() {
        if let Some(row) = rows.get(i)
            && row.key() == id
        {
            return Some(i);
        }
    }
    None
}

/// The first row with `id`.
#[cfg_attr(creusot, ensures(match result {
    Some(row) => found(rows@, id) && forall<i: Int> first_at(rows@, id, i) ==> *row == rows@[i],
    None => absent(rows@, id) && !found(rows@, id),
}))]
#[must_use]
pub fn lookup<T: Row>(rows: &[T], id: u32) -> Option<&T> {
    match index_of(rows, id) {
        Some(i) => rows.get(i),
        None => None,
    }
}
