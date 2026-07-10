use std::{error::Error, fmt};

use crate::{BlockId, EffectId, NodeId, SourceId, SymbolId, ValueId};

/// Result returned by checked graph construction operations.
pub type FlowResult<T> = Result<T, FlowError>;

/// Invalid input passed to a graph construction operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowError {
    /// An ID refers to no source in this graph.
    UnknownSource(SourceId),
    /// An ID refers to no block in this graph.
    UnknownBlock(BlockId),
    /// An ID refers to no node in this graph.
    UnknownNode(NodeId),
    /// An ID refers to no value in this graph.
    UnknownValue(ValueId),
    /// An ID refers to no symbol in this graph.
    UnknownSymbol(SymbolId),
    /// An ID refers to no effect in this graph.
    UnknownEffect(EffectId),
    /// A graph entity count exceeded the representable ID space.
    CapacityExceeded(&'static str),
}

impl fmt::Display for FlowError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownSource(id) => write!(formatter, "unknown source {id:?}"),
            Self::UnknownBlock(id) => write!(formatter, "unknown block {id:?}"),
            Self::UnknownNode(id) => write!(formatter, "unknown node {id:?}"),
            Self::UnknownValue(id) => write!(formatter, "unknown value {id:?}"),
            Self::UnknownSymbol(id) => write!(formatter, "unknown symbol {id:?}"),
            Self::UnknownEffect(id) => write!(formatter, "unknown effect {id:?}"),
            Self::CapacityExceeded(kind) => write!(formatter, "too many {kind} entities"),
        }
    }
}

impl Error for FlowError {}

/// A broken internal graph invariant found by [`crate::FlowGraph::validate`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvariantViolation {
    /// The stored ID does not match the entity's position in its arena.
    ArenaIdMismatch(&'static str, u32),
    /// A node is missing from its owning block or appears more than once.
    NodeOwnership(NodeId),
    /// A control edge is not indexed exactly once at one of its endpoints.
    ControlAdjacency(&'static str, u32),
    /// A value's definition and definition edge disagree.
    ValueDefinition(ValueId),
    /// An effect edge has an invalid endpoint.
    EffectEndpoint(u32),
}

/// Collection of graph invariant violations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationErrors(Vec<InvariantViolation>);

impl ValidationErrors {
    pub(crate) fn new(errors: Vec<InvariantViolation>) -> Self {
        Self(errors)
    }

    /// All violations found in a single validation pass.
    pub fn violations(&self) -> &[InvariantViolation] {
        &self.0
    }
}

impl fmt::Display for ValidationErrors {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "flow graph has {} invariant violation(s)",
            self.0.len()
        )
    }
}

impl Error for ValidationErrors {}
