//! L2→L3 residual-operation registry.
//!
//! Legalization has already erased every currently supported input-specific
//! operation before this edge. Keeping the empty registry explicit prevents
//! conversion code from growing ad-hoc variant matches later.

/// The current L2→L3 conversion has no residual dialect operation.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ResidualOperations;

/// The complete residual-operation set for the current conversion edge.
pub const REGISTRY: ResidualOperations = ResidualOperations;
