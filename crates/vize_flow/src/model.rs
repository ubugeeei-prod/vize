use crate::{
    BlockId, ControlEdgeId, DataEdgeId, EffectEdgeId, EffectId, NodeId, Provenance, SourceId,
    SymbolId, ValueId,
};
use vize_carton::{String, source_anchor::SourceAnchor};

/// A logical source participating in one flow graph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Source {
    pub(crate) id: SourceId,
    pub(crate) name: String,
    pub(crate) anchor: Option<SourceAnchor>,
}

impl Source {
    /// Stable source ID within this graph.
    pub const fn id(&self) -> SourceId {
        self.id
    }

    /// Producer-supplied path, URI, or logical source name.
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Compilation-level identity retained independently of this graph's local ID.
    pub const fn anchor(&self) -> Option<SourceAnchor> {
        self.anchor
    }
}

/// A basic block containing an ordered sequence of nodes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub(crate) id: BlockId,
    pub(crate) provenance: Provenance,
    pub(crate) nodes: Vec<NodeId>,
    pub(crate) incoming: Vec<ControlEdgeId>,
    pub(crate) outgoing: Vec<ControlEdgeId>,
}

impl Block {
    pub const fn id(&self) -> BlockId {
        self.id
    }

    pub const fn provenance(&self) -> Provenance {
        self.provenance
    }

    pub fn nodes(&self) -> &[NodeId] {
        &self.nodes
    }
}

/// Frontend-neutral role of an operation in a block.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeKind {
    /// An ordinary computation whose detailed semantics live in producer facts.
    #[default]
    Operation,
    /// A merge point for control or values.
    Merge,
    /// A single-assignment value merge.
    Phi,
    /// The operation transfers control out of the block.
    Terminator(TerminatorKind),
}

/// Frontend-neutral terminator category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TerminatorKind {
    Jump,
    Branch,
    Return,
    Throw,
    Suspend,
    Unreachable,
}

/// An ordered operation inside a basic block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    pub(crate) id: NodeId,
    pub(crate) block: BlockId,
    pub(crate) kind: NodeKind,
    pub(crate) provenance: Provenance,
}

impl Node {
    pub const fn id(&self) -> NodeId {
        self.id
    }

    pub const fn block(&self) -> BlockId {
        self.block
    }

    pub const fn kind(&self) -> NodeKind {
        self.kind
    }

    pub const fn provenance(&self) -> Provenance {
        self.provenance
    }
}

/// Meaning of a possible control transfer.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ControlEdgeKind {
    #[default]
    Normal,
    TrueBranch,
    FalseBranch,
    LoopBack,
    Break,
    Continue,
    Return,
    Exception,
    /// Declares the entry of a nested function subgraph, not an invocation.
    FunctionEntry,
    /// Records an OXC CFG relationship that cannot execute from this block.
    Unreachable,
}

/// Directed control-flow edge between blocks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlEdge {
    pub(crate) id: ControlEdgeId,
    pub(crate) from: BlockId,
    pub(crate) to: BlockId,
    pub(crate) kind: ControlEdgeKind,
    pub(crate) provenance: Provenance,
}

impl ControlEdge {
    pub const fn id(&self) -> ControlEdgeId {
        self.id
    }

    pub const fn from(&self) -> BlockId {
        self.from
    }

    pub const fn to(&self) -> BlockId {
        self.to
    }

    pub const fn kind(&self) -> ControlEdgeKind {
        self.kind
    }

    pub const fn provenance(&self) -> Provenance {
        self.provenance
    }
}

/// A producer-identified symbol, independent of frontend symbol-table types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Symbol {
    pub(crate) id: SymbolId,
    pub(crate) provenance: Provenance,
}

impl Symbol {
    pub const fn id(&self) -> SymbolId {
        self.id
    }

    pub const fn provenance(&self) -> Provenance {
        self.provenance
    }
}

/// A value flowing between operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Value {
    pub(crate) id: ValueId,
    pub(crate) definition: Option<NodeId>,
    pub(crate) symbol: Option<SymbolId>,
    pub(crate) provenance: Provenance,
}

impl Value {
    pub const fn id(&self) -> ValueId {
        self.id
    }

    pub const fn definition(&self) -> Option<NodeId> {
        self.definition
    }

    pub const fn symbol(&self) -> Option<SymbolId> {
        self.symbol
    }

    pub const fn provenance(&self) -> Provenance {
        self.provenance
    }
}

/// A non-definition relationship between a value and an operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DataUseKind {
    Use,
    PhiInput,
    Mutation,
    Capture,
}

/// Complete data-edge category, including graph-created definitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DataEdgeKind {
    Definition,
    Use,
    PhiInput,
    Mutation,
    Capture,
}

impl From<DataUseKind> for DataEdgeKind {
    fn from(kind: DataUseKind) -> Self {
        match kind {
            DataUseKind::Use => Self::Use,
            DataUseKind::PhiInput => Self::PhiInput,
            DataUseKind::Mutation => Self::Mutation,
            DataUseKind::Capture => Self::Capture,
        }
    }
}

/// Directed incidence edge between a value and an operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataEdge {
    pub(crate) id: DataEdgeId,
    pub(crate) value: ValueId,
    pub(crate) node: NodeId,
    pub(crate) kind: DataEdgeKind,
    pub(crate) provenance: Provenance,
}

impl DataEdge {
    pub const fn id(&self) -> DataEdgeId {
        self.id
    }

    pub const fn value(&self) -> ValueId {
        self.value
    }

    pub const fn node(&self) -> NodeId {
        self.node
    }

    pub const fn kind(&self) -> DataEdgeKind {
        self.kind
    }

    pub const fn provenance(&self) -> Provenance {
        self.provenance
    }
}

/// Observable or ordering-relevant operation category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EffectKind {
    Read,
    Write,
    Call,
    Allocate,
    Deallocate,
    Throw,
    Await,
    Yield,
    Io,
    Unknown,
}

/// One effect performed by a node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Effect {
    pub(crate) id: EffectId,
    pub(crate) node: NodeId,
    pub(crate) kind: EffectKind,
    pub(crate) symbol: Option<SymbolId>,
    pub(crate) provenance: Provenance,
}

impl Effect {
    pub const fn id(&self) -> EffectId {
        self.id
    }

    pub const fn node(&self) -> NodeId {
        self.node
    }

    pub const fn kind(&self) -> EffectKind {
        self.kind
    }

    pub const fn symbol(&self) -> Option<SymbolId> {
        self.symbol
    }

    pub const fn provenance(&self) -> Provenance {
        self.provenance
    }
}

/// Relationship between two effects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EffectEdgeKind {
    Order,
    Dependency,
    Conflict,
}

/// Directed relationship between effect facts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectEdge {
    pub(crate) id: EffectEdgeId,
    pub(crate) from: EffectId,
    pub(crate) to: EffectId,
    pub(crate) kind: EffectEdgeKind,
    pub(crate) provenance: Provenance,
}

impl EffectEdge {
    pub const fn id(&self) -> EffectEdgeId {
        self.id
    }

    pub const fn from(&self) -> EffectId {
        self.from
    }

    pub const fn to(&self) -> EffectId {
        self.to
    }

    pub const fn kind(&self) -> EffectEdgeKind {
        self.kind
    }

    pub const fn provenance(&self) -> Provenance {
        self.provenance
    }
}
